use bitbucket::{get_current_branch, get_current_repo_id};
use std::process::{Command, Stdio};
use url::Url;

pub struct Browse;

impl Browse {
    pub fn handle(_args: std::env::Args) -> Result<(), ()> {
        let repo_id = get_current_repo_id().ok_or(())?;
        let branch = get_current_branch().ok_or(())?;

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
            .map_err(|_| ())
    }
}
