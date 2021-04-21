mod repo_id;
pub use repo_id::RepoId;

mod client;
pub use client::Client;

mod pull_request;
pub use pull_request::PullRequest;

use git2::Repository;

pub fn get_current_repo_id(repo: &Repository) -> Option<RepoId> {
    let remotes = repo.remotes().ok()?;

    remotes.iter().find_map(|remote| {
        let remote = repo.find_remote(remote.unwrap()).unwrap();
        RepoId::from_url(remote.url().unwrap())
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
