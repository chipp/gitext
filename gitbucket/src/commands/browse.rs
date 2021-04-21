use bitbucket::{get_current_branch, get_current_repo_id};
use git2::Repository;
use std::process::{Command, Stdio};
use url::Url;

pub struct Browse;

impl Browse {
    pub fn handle(_args: std::env::Args, repo: Repository) -> Result<(), String> {
        let repo_id =
            get_current_repo_id(&repo).ok_or(String::from("this is not a bitbucket repository"))?;
        let branch =
            get_current_branch(&repo).ok_or(String::from("can't find the current branch"))?;

        let mut url = Url::parse(&repo_id.url()).unwrap();

        {
            let mut segments = url.path_segments_mut().unwrap();
            segments.push("browse");
        }

        url.query_pairs_mut().append_pair("at", &branch);

        Command::new("open")
            .arg(url.as_str())
            .stdout(Stdio::null())
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("can't open URL {}: {}", url, e))
    }
}
