mod commands;
use commands::*;

mod error;
pub use error::Error;

use common_git::{get_config, get_repo};
use std::env::Args;

pub async fn handle(args: Args) -> Result<(), Error> {
    let mut args = args;
    let _ = args.next();

    let path = std::env::var("REPO_PATH").unwrap_or(".".to_string());
    let path = std::fs::canonicalize(path).unwrap();
    let path = path.to_str().unwrap();

    let repo = get_repo(path)?;
    let config = get_config(&repo)?;

    match args.next().as_ref().map(String::as_str) {
        Some("browse") => Browse::handle(args, repo, config, &path).await,
        Some("ticket") => Ticket::handle(args, repo, config).await,
        Some("pr") => Pr::handle(args, repo, config).await,
        Some("prs") => Prs::handle(args, repo, config).await,
        Some("auth") => Auth::handle(args, config).await,
        Some(command) => Err(Error::UnknownCommand(command.to_string())),
        None => {
            // TODO: help message
            panic!()
        }
    }
}
