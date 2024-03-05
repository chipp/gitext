mod commands {
    pub mod ticket;
}

use std::path::Path;
use std::process::{exit, Command};

use clap::ArgMatches;
pub use commands::ticket::Ticket;

mod cli;
mod common_git;
mod error;
mod jira;
mod shellquote;

mod bitbucket;
mod gitbucket;

mod gitlab;
mod gitlad;

mod gighub;
mod github;

use git2::{Config as GitConfig, Repository};
use git2::{ErrorClass as GitErrorClass, ErrorCode as GitErrorCode};
use url::Url;

use cli::cli;
use common_git::{
    get_aliases_from_config, get_config, get_repo, set_provider, Config, ConfigError, Provider::*,
};
use error::Error;

use clap::error::ErrorKind as ClapErrorKind;

use crate::common_git::Provider;

type Result<T> = std::result::Result<T, Error>;

pub async fn handle(args: std::env::Args) -> Result<()> {
    let path = std::env::var("REPO_PATH").unwrap_or(".".to_string());
    let path = std::fs::canonicalize(path).unwrap();

    let mut args = args.collect::<Vec<_>>();

    let (repo, config) = match repo_and_config(&path) {
        Ok(tuple) => tuple,
        Err(Error::Git(err))
            if err.class() == GitErrorClass::Repository && err.code() == GitErrorCode::NotFound =>
        {
            let matches = match cli(Provider::GitHub).try_get_matches_from(&args) {
                Ok(matches) => matches,
                Err(_) => {
                    return exec_git_cmd(&args[1..], &path);
                }
            };

            if let Some(("clone", args)) = matches.subcommand() {
                let config = Config::default();
                return handle_github_clone(args, &config, &path).await;
            } else {
                return exec_git_cmd(&args[1..], &path);
            }
        }
        Err(_) => {
            return exec_git_cmd(&args[1..], &path);
        }
    };

    let matches = match cli(config.provider).try_get_matches_from(&args) {
        Ok(matches) => matches,
        Err(err) => match err.kind() {
            ClapErrorKind::InvalidSubcommand | ClapErrorKind::UnknownArgument => {
                resolve_alias(&path, &mut args)?;

                match cli(config.provider).try_get_matches_from(&args) {
                    Ok(matches) => matches,
                    Err(_) => return exec_git_cmd(&args[1..], &path),
                }
            }
            _ => {
                err.exit();
            }
        },
    };

    let (command, sub_matches) = matches.subcommand().unwrap();

    let is_handled = match config.provider {
        BitBucket => handle_bitbucket(&command, sub_matches, &repo, &config, &path).await?,
        GitLab => handle_gitlab(&command, sub_matches, &repo, &config, &path).await?,
        GitHub => handle_github(&command, sub_matches, &repo, &config, &path).await?,
    };

    if !is_handled {
        match command.as_ref() {
            "ticket" => Ticket::handle(repo, config)?,
            _ => exec_git_cmd(&args[1..], &path)?,
        }
    }

    Ok(())
}

fn repo_and_config(path: &Path) -> Result<(Repository, Config)> {
    let repo = get_repo(&path)?;

    let config = match get_config(&repo) {
        Ok(config) => config,
        Err(ConfigError::ProviderNotSpecified) => {
            if let Some(true) = is_github_repo(&repo) {
                set_provider(&repo, GitHub)?;
                get_config(&repo)?
            } else {
                return Err(ConfigError::ProviderNotSpecified.into());
            }
        }
        Err(err) => return Err(err.into()),
    };

    Ok((repo, config))
}

fn is_github_repo(repo: &Repository) -> Option<bool> {
    let remote = repo.find_remote("origin").ok()?;

    let base_url = Url::parse("https://github.com").unwrap();
    let _ = github::RepoId::from_str_with_host(remote.url().unwrap(), &base_url).ok()?;

    Some(true)
}

fn resolve_alias(path: &Path, args: &mut Vec<String>) -> Result<()> {
    let config = if let Ok(repo) = get_repo(&path) {
        repo.config()?
    } else {
        GitConfig::open_default()?
    };

    let aliases = get_aliases_from_config(&config);

    if let Some(resolved) = aliases.get(&args[1]) {
        args.remove(1);

        let resolved = shellquote::split(&resolved).collect::<Vec<_>>();
        for (index, result) in resolved.into_iter().enumerate() {
            let value = result.map_err(|err| Error::InvalidAlias(args[1].clone(), err))?;
            args.insert(index + 1, value);
        }
    }

    Ok(())
}

async fn handle_bitbucket(
    command: &str,
    args: &ArgMatches,
    repo: &Repository,
    config: &Config,
    path: &Path,
) -> Result<bool> {
    use gitbucket::{Auth, Browse, Create, Pr, Prs, Switch};

    match command {
        "auth" => Auth::handle(config).await?,
        "browse" => Browse::handle(args, repo, config, &path)?,
        "create" => Create::handle(args, repo, config).await?,
        "pr" => Pr::handle(args, repo, config).await?,
        "prs" => Prs::handle(args, repo, config).await?,
        "switch" => {
            if !Switch::handle(args, repo, config).await? {
                return Ok(false);
            }
        }
        _ => return Ok(false),
    }

    Ok(true)
}

async fn handle_gitlab(
    command: &str,
    args: &ArgMatches,
    repo: &Repository,
    config: &Config,
    path: &Path,
) -> Result<bool> {
    use gitlad::{Auth, Browse, Pr, Prs, Switch};

    match command {
        "auth" => Auth::handle(config).await?,
        "browse" => Browse::handle(args, repo, config, &path)?,
        "create" => unimplemented!("to be implemented"),
        "pr" => Pr::handle(args, repo, config).await?,
        "prs" => Prs::handle(args, repo, config).await?,
        "switch" => {
            if !Switch::handle(args, repo, config).await? {
                return Ok(false);
            }
        }
        _ => return Ok(false),
    }

    Ok(true)
}

async fn handle_github(
    command: &str,
    args: &ArgMatches,
    repo: &Repository,
    config: &Config,
    path: &Path,
) -> Result<bool> {
    use gighub::{Auth, Browse, Create, Pr, Prs, Switch};

    match command {
        "auth" => Auth::handle(config).await?,
        "browse" => Browse::handle(args, repo, config, &path)?,
        "create" => Create::handle(args, repo, config).await?,
        "pr" => Pr::handle(args, repo, config).await?,
        "prs" => Prs::handle(repo, config).await?,
        "switch" => {
            if !Switch::handle(args, repo, config).await? {
                return Ok(false);
            }
        }
        _ => return Ok(false),
    }

    Ok(true)
}

async fn handle_github_clone(args: &ArgMatches, config: &Config, path: &Path) -> Result<()> {
    gighub::Clone::handle(args, config, path).await
}

fn exec_git_cmd(args: &[String], path: &Path) -> Result<()> {
    let mut git = Command::new("git");

    if let Ok(repo) = get_repo(&path) {
        git.arg(format!("--git-dir={}", repo.path().to_string_lossy()));

        if let Some(workdir) = repo.workdir() {
            git.arg(format!("--work-tree={}", workdir.to_string_lossy()));
        }
    }

    let git = git.args(args);

    let output = git
        .spawn()
        .expect("failed to execute process")
        .wait()
        .map_err(Error::FailedToExecuteGit)?;
    if !output.success() {
        exit(output.code().unwrap_or(-1));
    }

    Ok(())
}
