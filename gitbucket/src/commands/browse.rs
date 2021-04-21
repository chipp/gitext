use bitbucket::{get_current_branch, get_current_repo_id};
use git2::Repository;
use std::process::{Command, Stdio};

pub struct Browse;

impl Browse {
    pub async fn handle(_args: std::env::Args, repo: Repository, path: &str) -> Result<(), String> {
        let repo_id =
            get_current_repo_id(&repo).ok_or(String::from("this is not a bitbucket repository"))?;
        let branch =
            get_current_branch(&repo).ok_or(String::from("can't find the current branch"))?;

        let mut url = repo_id.url();

        {
            let mut segments = url.path_segments_mut().unwrap();
            segments.push("browse");

            let current_path = std::path::Path::new(path);
            let relative_path = repo
                .workdir()
                .map(|p| current_path.strip_prefix(&p).ok())
                .flatten();

            if let Some(relative_path) = relative_path {
                for comp in relative_path.components().map(|comp| comp.as_os_str()) {
                    if let Some(comp) = comp.to_str() {
                        segments.push(comp);
                    }
                }
            }
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
