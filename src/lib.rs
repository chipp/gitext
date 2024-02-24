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

use cli::cli;
use common_git::{get_aliases_from_config, get_config, get_repo, Config, Provider::*};
use error::Error;

type Result<T> = std::result::Result<T, Error>;

const SUPPORTED_COMMANDS: &[&str] = &["auth", "browse", "create", "pr", "prs", "switch", "ticket"];

pub async fn handle(args: std::env::Args) -> Result<()> {
    let path = std::env::var("REPO_PATH").unwrap_or(".".to_string());
    let path = std::fs::canonicalize(path).unwrap();

    let mut args = args.collect::<Vec<_>>();

    let mut matches = cli().get_matches_from(&args);
    let (mut command, mut sub_matches) = matches.subcommand().unwrap();

    if !SUPPORTED_COMMANDS.contains(&command) {
        resolve_alias(&path, &mut args)?;

        matches = cli().get_matches_from(&args);
        (command, sub_matches) = matches.subcommand().unwrap();
    }

    if !SUPPORTED_COMMANDS.contains(&command) {
        return exec_git_cmd(&args[1..], &path);
    }

    let (repo, config) = match repo_and_config(&path) {
        Ok(tuple) => tuple,
        Err(_) => return exec_git_cmd(&args[1..], &path),
    };

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
    let config = get_config(&repo)?;

    Ok((repo, config))
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
    use gighub::{Auth, Browse, Pr, Prs, Switch};

    match command {
        "auth" => Auth::handle(config).await?,
        "browse" => Browse::handle(args, repo, config, &path)?,
        "create" => unimplemented!("to be implemented"),
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
