mod commands;
use commands::*;

use git2::{Repository, RepositoryOpenFlags};
use std::env::Args;

pub async fn handle(args: Args) -> Result<(), String> {
    let mut args = args;
    let _ = args.next();

    let path = ".";

    match args.next().as_ref().map(String::as_str) {
        Some("browse") => Browse::handle(args, get_repo(path)?).await,
        Some("ticket") => Ticket::handle(args, get_repo(path)?).await,
        Some("pr") => Pr::handle(args, get_repo(path)?).await,
        Some(command) => return Err(format!("unknown command {}", command)),
        None => return Err(String::from("no command")),
    }
}

fn get_repo(path: &str) -> Result<Repository, String> {
    Repository::open_ext(
        path,
        RepositoryOpenFlags::empty(),
        vec![dirs::home_dir().unwrap()],
    )
    .map_err(|e| format!("failed to open: {}", e))
}
