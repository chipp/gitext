#![allow(dead_code)]
#![allow(unused_imports)]

mod repo_id;

pub use repo_id::RepoId;

mod client;
pub use client::Client;

mod pull_request;
pub use pull_request::{PullRequest, PullRequestState};

mod pipeline;
pub use pipeline::{Pipeline, PipelineStatus};

mod user;

use git2::{Remote, Repository};

use crate::git::BaseUrlConfig;

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

pub fn get_gitlab_remote<'r, 'c, Conf>(repo: &'r Repository, config: &'c Conf) -> Option<Remote<'r>>
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
