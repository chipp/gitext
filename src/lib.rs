mod commands {
    pub mod ticket;
}

use std::path::{Path, PathBuf};
use std::process::{exit, Command};

pub use commands::ticket::Ticket;

mod common_git;
mod error;
mod jira;
mod split_once;

mod bitbucket;
mod gitbucket;

mod gitlab;
mod gitlad;

use git2::Repository;

use common_git::{get_config, get_repo, Config, Provider::*};
use error::Error;

use crate::gitlad::Switch;

pub type ErasedError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, ErasedError>;

const SUPPORTED_COMMANDS: &[&str] = &["auth", "browse", "pr", "prs", "switch"];

pub async fn handle(args: std::env::Args) -> Result<()> {
    let mut args = args;
    let _ = args.next();

    let path = std::env::var("REPO_PATH").unwrap_or(".".to_string());
    let path = std::fs::canonicalize(path).unwrap();

    let repo = get_repo(&path)?;
    let config = get_config(&repo)?;

    let mut args = args.collect::<Vec<_>>();

    let command = args
        .first()
        .map(Clone::clone)
        .expect("here should be help message ¯\\_(ツ)_/¯");

    if !SUPPORTED_COMMANDS.contains(&command.as_str()) {
        if let Some(resolved) = config.aliases.get(&command) {
            args.remove(0);

            for component in resolved.rsplit(" ") {
                args.insert(0, component.to_string());
            }
        }
    }

    let command = args
        .first()
        .map(String::as_str)
        .expect("here should be help message ¯\\_(ツ)_/¯");

    let is_handled = match config.provider {
        BitBucket => handle_bitbucket(&command, &args[1..], &repo, &config, &path).await?,
        GitLab => handle_gitlab(&command, &args[1..], &repo, &config, &path).await?,
    };

    if !is_handled {
        match command.as_ref() {
            "ticket" => Ticket::handle(repo, config)?,
            _ => exec_git_cmd(args, &path)?,
        }
    }

    Ok(())
}

async fn handle_bitbucket<Arg: AsRef<str>>(
    command: &str,
    args: &[Arg],
    repo: &Repository,
    config: &Config,
    path: &Path,
) -> Result<bool> {
    use gitbucket::{Auth, Browse, Pr, Prs};

    match command {
        "auth" => Auth::handle(config).await?,
        "browse" => Browse::handle(args, repo, config, &path)?,
        "pr" => Pr::handle(args, repo, config).await?,
        "prs" => Prs::handle(args, repo, config).await?,
        _ => return Ok(false),
    }

    Ok(true)
}

async fn handle_gitlab<Arg: AsRef<str>>(
    command: &str,
    args: &[Arg],
    repo: &Repository,
    config: &Config,
    path: &Path,
) -> Result<bool> {
    use gitlad::{Auth, Browse, Pr, Prs};

    match command {
        "auth" => Auth::handle(config).await?,
        "browse" => Browse::handle(args, repo, config, &path)?,
        "switch" => {
            if !Switch::handle(args, repo, config).await? {
                return Ok(false);
            }
        }
        "pr" => Pr::handle(args, repo, config).await?,
        "prs" => Prs::handle(args, repo, config).await?,
        _ => return Ok(false),
    }

    Ok(true)
}

fn exec_git_cmd(args: Vec<String>, path: &Path) -> Result<()> {
    let worktree = path.to_string_lossy();

    let mut path = PathBuf::from(path);
    path.push(".git");
    let dot_git = path.to_string_lossy();

    let mut git = Command::new("git");

    let git = git
        .arg(format!("--git-dir={}", dot_git))
        .arg(format!("--work-tree={}", worktree))
        .args(args);

    let output = git.spawn().expect("failed to execute process").wait()?;
    if !output.success() {
        exit(output.code().unwrap_or(-1));
    }

    Ok(())
}
