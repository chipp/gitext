mod client;
mod pull_request;
mod repo_id;
mod split_once;
mod user;

pub use client::Client;
pub use pull_request::PullRequest;
pub use repo_id::RepoId;

use std::str::FromStr;

use git2::{Error as GitError, Remote, Repository, RepositoryOpenFlags};

pub fn get_repo(path: &str) -> Result<Repository, GitError> {
    Repository::open_ext(
        path,
        RepositoryOpenFlags::empty(),
        vec![dirs::home_dir().unwrap()],
    )
}

pub fn get_current_repo_id(repo: &Repository) -> Option<RepoId> {
    let remotes = repo.remotes().ok()?;

    remotes.iter().find_map(|remote| {
        let remote = repo.find_remote(remote.unwrap()).unwrap();
        RepoId::from_str(remote.url().unwrap()).ok()
    })
}

pub fn get_bitbucket_remote(repo: &Repository) -> Option<Remote> {
    let remotes = repo.remotes().ok()?;

    remotes.iter().find_map(|remote| {
        let remote = repo.find_remote(remote.unwrap()).unwrap();
        if RepoId::from_str(remote.url().unwrap()).is_ok() {
            Some(remote)
        } else {
            None
        }
    })
}

pub fn get_current_branch(repo: &Repository) -> Option<String> {
    let head = repo.head().ok()?;

    if head.is_branch() {
        head.name().map(String::from)
    } else {
        None
    }
}
