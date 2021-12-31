mod client;
mod pull_request;
mod repo_id;
mod split_once;
mod user;

pub use client::Client;
pub use pull_request::PullRequest;
pub use repo_id::RepoId;

use common_git::BaseUrlConfig;
use git2::{Remote, Repository};

pub fn get_current_repo_id(repo: &Repository, config: &dyn BaseUrlConfig) -> Option<RepoId> {
    let remotes = repo.remotes().ok()?;

    remotes.iter().find_map(|remote| {
        let remote = repo.find_remote(remote.unwrap()).unwrap();
        RepoId::from_str_with_host(remote.url().unwrap(), config.base_url()).ok()
    })
}

pub fn get_bitbucket_remote<'r, 'c>(
    repo: &'r Repository,
    config: &'c dyn BaseUrlConfig,
) -> Option<Remote<'r>> {
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
