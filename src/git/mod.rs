mod config;

#[cfg(feature = "git-cli")]
mod git_cli {
    pub mod branch;
    pub mod clone;
    pub mod fetch;
}

#[cfg(feature = "git-cli")]
pub use git_cli::branch::switch_to_branch;

#[cfg(feature = "git-cli")]
pub use git_cli::clone::clone_repo;

#[cfg(feature = "git-cli")]
pub use git_cli::fetch::fetch_remote;

#[cfg(not(feature = "git-cli"))]
mod libgit2 {
    pub mod branch;
    pub mod clone;
    pub mod fetch;

    mod credential_helper;
}

#[cfg(not(feature = "git-cli"))]
pub use libgit2::branch::switch_to_branch;

#[cfg(not(feature = "git-cli"))]
pub use libgit2::clone::clone_repo;

#[cfg(not(feature = "git-cli"))]
pub use libgit2::fetch::fetch_remote;

use std::ffi::OsStr;
use std::path::Path;
use std::process::{exit, Command};

pub use config::{
    get_aliases_from_config, get_config, set_config, set_provider, Config, ConfigError, Provider,
};
pub use config::{AuthDomainConfig, BaseUrlConfig, JiraAuthDomainConfig, JiraUrlConfig};

use git2::{Branch, BranchType, Remote, RepositoryOpenFlags};
use git2::{Error as GitError, Repository};
use regex::Regex;

use super::Error;

pub fn get_repo(path: &Path) -> Result<Repository, GitError> {
    Repository::open_ext(
        path,
        RepositoryOpenFlags::empty(),
        vec![dirs::home_dir().unwrap()],
    )
}

pub fn get_current_branch(repo: &Repository) -> Option<String> {
    let head = repo.head().ok()?;

    if head.is_branch() {
        head.name()
            .map(|name| name.trim_start_matches("refs/heads/"))
            .map(String::from)
    } else {
        None
    }
}

pub fn find_remote_branch<'repo>(
    branch_name: &str,
    remote: &Remote,
    repo: &'repo Repository,
) -> Result<Branch<'repo>, GitError> {
    let remote_name = remote.name().unwrap();
    let branch_name = format!("{}/{}", remote_name, branch_name);

    let branch = repo.find_branch(&branch_name, BranchType::Remote)?;

    println!("found remote branch {}", branch.name().unwrap().unwrap());

    Ok(branch)
}

pub fn extract_ticket<'b>(branch: &'b str) -> Option<&'b str> {
    let re = Regex::new(r"\w{2,}-\d+").unwrap();
    re.captures(&branch)
        .map(|caps| caps.get(0).unwrap().as_str())
}

pub fn exec_git_cmd<A, I>(args: I, repo: Option<&Repository>) -> Result<(), Error>
where
    A: AsRef<OsStr>,
    I: IntoIterator<Item = A>,
{
    let mut git = Command::new("git");

    if let Some(repo) = repo {
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
