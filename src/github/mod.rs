#![allow(dead_code)]
#![allow(unused_imports)]

mod check_suite;
mod client;
mod pull_request;
mod repo;
mod repo_id;
mod user;

pub use check_suite::{CheckSuites, Conclusion, Status};
pub use client::Client;
pub use pull_request::{PullRequest, State};
pub use repo_id::RepoId;

use crate::git::BaseUrlConfig;
use git2::{Remote, Repository};

pub fn get_current_repo_id<Conf>(repo: &Repository, config: &Conf) -> Option<RepoId>
where
    Conf: BaseUrlConfig,
{
    let remotes = repo.remotes().ok()?;

    remotes.iter().find_map(|remote| {
        let remote = repo.find_remote(remote.unwrap()).unwrap();
        RepoId::from_str_with_host(remote.url().unwrap(), config.base_url()).ok()
    })
}

pub fn get_github_remote<'r, 'c, Conf>(repo: &'r Repository, config: &'c Conf) -> Option<Remote<'r>>
where
    Conf: BaseUrlConfig,
{
    let remotes = repo.remotes().ok()?;

    remotes.iter().find_map(|remote| {
        let remote = repo.find_remote(remote.unwrap()).unwrap();
        if RepoId::from_str_with_host(remote.url().unwrap(), config.base_url()).is_ok() {
            Some(remote)
        } else {
            None
        }
    })
}
