mod check_suite;
mod client;
mod pull_request;
mod repo_id;
mod user;

pub use check_suite::{CheckSuites, Conclusion, Status};
pub use client::Client;
pub use pull_request::PullRequest;
pub use repo_id::RepoId;

use crate::common_git::BaseUrlConfig;
use git2::Repository;

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
