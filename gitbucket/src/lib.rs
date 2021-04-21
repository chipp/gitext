mod commands;
use commands::*;

pub fn handle(args: std::env::Args) -> Result<(), ()> {
    let mut args = args;
    let _ = args.next();

    match args.next().as_ref().map(String::as_str) {
        Some("browse") => Browse::handle(args),
        Some(command) => {
            eprintln!("unknown command {}", command);
            Err(())
        }
        None => Err(()),
    }
}
