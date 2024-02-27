use crate::common_git::{AuthDomainConfig, BaseUrlConfig};
use crate::github::{Client, RepoId};
use crate::Error;

use clap::ArgMatches;
use git2::Repository;

pub struct Create;

impl Create {
    pub async fn handle<Conf>(
        args: &ArgMatches,
        repo: &Repository,
        config: &Conf,
    ) -> Result<(), Error>
    where
        Conf: AuthDomainConfig + Send + Sync,
        Conf: BaseUrlConfig,
    {
        let org = args.get_one::<String>("org").map(String::as_str);
        let private = args.get_flag("private");

        // TODO: map errors correctly
        let name = repo
            .workdir()
            .ok_or(Error::NotInWorkTree)?
            .file_name()
            .ok_or(Error::NotInWorkTree)?
            .to_os_string()
            .into_string()
            .map_err(|_| Error::NotInWorkTree)?;

        let client = Client::new(config);

        let owner = org.unwrap_or(&client.whoami().await?.login).to_string();

        let repo_id = RepoId {
            owner: owner.clone(),
            repo: name.clone(),
        };

        let repository = if let Ok(repository) = client.get_repo(&repo_id).await {
            if !repository.private && private {
                return Err(Error::RepoExistsAndPublic(format!("{owner}/{name}")));
            }

            println!("Found existing repository");

            repository
        } else if let Some(org) = org {
            client.create_org_repo(org, &name, private).await?
        } else {
            client.create_user_repo(&name, private).await?
        };

        let remote_name = args.get_one::<String>("remote-name").unwrap();
        if let Ok(remote) = repo.find_remote(remote_name) {
            let url = remote.url().unwrap();
            let remote_repo_id = RepoId::from_str_with_host(url, &config.base_url())
                .expect(&format!("`{url}` should be valid http/git/ssh/ url"));

            if remote_repo_id != repo_id {
                return Err(Error::RemoteExists(
                    remote_name.to_string(),
                    remote.url().unwrap().to_string(),
                ));
            }
        } else {
            repo.remote(remote_name, &repository.ssh_url)?;
            println!("updated remote `{remote_name}`");
        }

        println!("{}", repository.html_url);

        Ok(())
    }
}
