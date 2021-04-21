mod commands;
use commands::*;

use bitbucket::get_repo;
use std::env::Args;

pub async fn handle(args: Args) -> Result<(), String> {
    let mut args = args;
    let _ = args.next();

    let path = std::fs::canonicalize(".").map_err(|err| format!("{}", err))?;
    let path = path.to_str().unwrap();

    match args.next().as_ref().map(String::as_str) {
        Some("browse") => Browse::handle(args, get_repo(path)?, &path).await,
        Some("ticket") => Ticket::handle(args, get_repo(path)?).await,
        Some("pr") => Pr::handle(args, get_repo(path)?).await,
        Some(command) => return Err(format!("unknown command {}", command)),
        None => return Err(String::from("no command")),
    }
}
