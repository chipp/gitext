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

use crate::git::{find_remote_by_priority, BaseUrlConfig};

pub fn get_current_repo_id<Conf>(repo: &Repository, config: &Conf) -> Option<RepoId>
where
    Conf: BaseUrlConfig,
{
    find_remote_by_priority(repo, |remote| {
        RepoId::from_str_with_host(remote.url()?, config.base_url()).ok()
    })
}

pub fn get_gitlab_remote<'r, 'c, Conf>(repo: &'r Repository, config: &'c Conf) -> Option<Remote<'r>>
where
    Conf: BaseUrlConfig,
{
    find_remote_by_priority(repo, |remote| {
        if RepoId::from_str_with_host(remote.url()?, config.base_url()).is_ok() {
            Some(remote)
        } else {
            None
        }
    })
}
