use std::path::Path;
use std::process::{Command, Stdio};

use crate::bitbucket::get_current_repo_id;
use crate::git::{get_current_branch, BaseUrlConfig};
use crate::Error;

use clap::ArgMatches;
use git2::Repository;

pub struct Browse;

impl Browse {
    pub fn handle<Conf>(
        args: &ArgMatches,
        repo: &Repository,
        config: &Conf,
        path: &Path,
    ) -> Result<(), Error>
    where
        Conf: BaseUrlConfig,
    {
        let repo_id = get_current_repo_id(&repo, config).ok_or(Error::InvalidRepo)?;
        let branch = get_current_branch(&repo).ok_or(Error::Detached)?;

        let mut url = repo_id.url(&config.base_url());

        let command = args.subcommand().unwrap_or(("repo", args));

        {
            let mut segments = url.path_segments_mut().unwrap();

            match command {
                ("pr", args) => {
                    let id: u16 = *args.get_one("id").expect("required");
                    segments.push("pull-requests");
                    segments.push(&format!("{}", id));
                }
                ("repo", _) => {
                    segments.push("browse");

                    let relative_path =
                        repo.workdir().map(|p| path.strip_prefix(&p).ok()).flatten();

                    if let Some(relative_path) = relative_path {
                        for comp in relative_path.components().map(|comp| comp.as_os_str()) {
                            if let Some(comp) = comp.to_str() {
                                segments.push(comp);
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        }

        if command.0 == "repo" {
            url.query_pairs_mut().append_pair("at", &branch);
        }

        Command::new("open")
            .arg(url.as_str())
            .stdout(Stdio::null())
            .spawn()
            .map(|_| ())
            .map_err(|err| Error::OpenUrl(err, url))
    }
}
