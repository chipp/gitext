mod commands {
    pub mod ticket;
}

pub use commands::ticket::Ticket;

mod common_git;
mod error;
mod jira;
mod split_once;

mod bitbucket;
mod gitbucket;

mod gitlab;
mod gitlad;

mod gighub;
mod github;

pub use error::Error;

use common_git::{get_config, get_repo, Provider::*};
use std::env::Args;

pub async fn handle(args: Args) -> Result<(), Error> {
    let mut args = args;
    let _ = args.next();

    let path = std::env::var("REPO_PATH").unwrap_or(".".to_string());
    let path = std::fs::canonicalize(path).unwrap();
    let path = path.to_str().unwrap();

    let repo = get_repo(path)?;
    let config = get_config(&repo)?;

    match (args.next().as_ref().map(String::as_str), &config.provider) {
        (Some("browse"), BitBucket) => gitbucket::Browse::handle(args, repo, config, &path).await,
        (Some("auth"), BitBucket) => gitbucket::Auth::handle(args, config).await,
        (Some("pr"), BitBucket) => gitbucket::Pr::handle(args, repo, config).await,
        (Some("prs"), BitBucket) => gitbucket::Prs::handle(args, repo, config).await,

        (Some("browse"), GitLab) => gitlad::Browse::handle(args, repo, config, &path).await,
        (Some("auth"), GitLab) => gitlad::Auth::handle(args, config).await,
        (Some("pr"), GitLab) => gitlad::Pr::handle(args, repo, config).await,
        (Some("prs"), GitLab) => gitlad::Prs::handle(args, repo, config).await,

        (Some("browse"), GitHub) => gighub::Browse::handle(args, repo, config, &path).await,
        (Some("auth"), GitHub) => gighub::Auth::handle(args, config).await,
        (Some("prs"), GitHub) => gighub::Prs::handle(args, repo, config).await,

        (Some("ticket"), _) => Ticket::handle(args, repo, config).await,
        (Some(command), _) => Err(Error::UnknownCommand(command.to_string())),
        (None, _) => {
            // TODO: help message
            panic!()
        }
    }
}
