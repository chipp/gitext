mod commands;
use commands::*;

mod error;
pub use error::Error;

use bitbucket::get_repo;
use std::env::Args;

pub async fn handle(args: Args) -> Result<(), Error> {
    let mut args = args;
    let _ = args.next();

    let path = std::fs::canonicalize(".").unwrap();
    let path = path.to_str().unwrap();

    match args.next().as_ref().map(String::as_str) {
        Some("browse") => Browse::handle(args, get_repo(path)?, &path).await,
        Some("ticket") => Ticket::handle(args, get_repo(path)?).await,
        Some("pr") => Pr::handle(args, get_repo(path)?).await,
        Some(command) => Err(Error::UnknownCommand(command.to_string())),
        None => {
            // TODO: help message
            panic!()
        }
    }
}
