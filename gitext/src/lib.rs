mod commands {
    pub mod ticket;
}

pub use commands::ticket::Ticket;

use common_git::{get_config, get_repo, Provider::*};
use error::Error;
use gitbucket::{
    Auth as GitBucketAuth, Browse as GitBucketBrowse, Pr as GitBucketPr, Prs as GitBucketPrs,
};
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
        (Some("browse"), BitBucket) => GitBucketBrowse::handle(args, repo, config, &path).await,
        (Some("auth"), BitBucket) => GitBucketAuth::handle(args, config).await,
        (Some("pr"), BitBucket) => GitBucketPr::handle(args, repo, config).await,
        (Some("prs"), BitBucket) => GitBucketPrs::handle(args, repo, config).await,

        (Some("ticket"), _) => Ticket::handle(args, repo, config).await,
        (Some(command), _) => Err(Error::UnknownCommand(command.to_string())),
        (None, _) => {
            // TODO: help message
            panic!()
        }
    }
}
