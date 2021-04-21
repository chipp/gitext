mod commands;
use commands::*;

use git2::{Repository, RepositoryOpenFlags};

pub fn handle(args: std::env::Args) -> Result<(), String> {
    let mut args = args;
    let _ = args.next();

    let path = ".";

    match args.next().as_ref().map(String::as_str) {
        Some("browse") => Browse::handle(args, get_repo(path)?),
        Some("ticket") => Ticket::handle(args, get_repo(path)?),
        Some(command) => Err(format!("unknown command {}", command)),
        None => Err(String::from("no command")),
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
