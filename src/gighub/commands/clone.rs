use std::path::Path;

use clap::ArgMatches;

use crate::error::Error;
use crate::git::{clone_repo, set_config, Config};
use crate::github::{Client, RepoId};

pub struct Clone;

impl Clone {
    pub async fn handle(args: &ArgMatches, config: &Config, path: &Path) -> Result<(), Error> {
        let repo_id: &RepoId = args.get_one("repo").unwrap();

        let client = Client::new(config);
        let repository = client.get_repo(repo_id).await?;

        let mut path = path.to_path_buf();
        path.push(repo_id.repo.clone());

        let path = path.as_path();

        let repo = clone_repo(&repository.ssh_url, path, config)?;
        set_config(&repo, config)?;

        Ok(())
    }
}
