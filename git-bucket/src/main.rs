use bitbucket::*;

fn main() {
    match get_current_repo_id() {
        Some(repo_id) => println!("{}", repo_id.url()),
        None => {
            eprintln!("not a bitbucket repo");
            std::process::exit(1)
        }
    }
}
