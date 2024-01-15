use crate::bitbucket::{Client, RepoId};
use crate::common_git::{AuthDomainConfig, BaseUrlConfig};
use crate::Error;

use git2::Repository;

pub struct Create;

impl Create {
    pub async fn handle<Arg: AsRef<str>, Conf>(
        args: &[Arg],
        repo: &Repository,
        config: &Conf,
    ) -> Result<(), Error>
    where
        Conf: AuthDomainConfig + Send + Sync,
        Conf: BaseUrlConfig,
    {
        let project = args
            .first()
            .ok_or(Error::MissingProjectCode)?
            .as_ref()
            .to_string();

        // TODO: map errors correctly
        let name = repo
            .workdir()
            .ok_or(Error::NotInWorkTree)?
            .file_name()
            .ok_or(Error::NotInWorkTree)?
            .to_os_string()
            .into_string()
            .map_err(|_| Error::NotInWorkTree)?;

        let repo_id = RepoId { project, name };

        let client = Client::new(config);
        let repository = client.create_repo(repo_id).await?;

        if let Some(link) = repository.links.self_.first() {
            println!("Create a repository {}", link.href);
        }

        let mut url = None;

        for link in &repository.links.clone {
            if let Some("ssh") = link.name.as_deref() {
                url = Some(link.href.as_str());
                break;
            }
        }

        if let None = url {
            url = repository
                .links
                .clone
                .first()
                .map(|link| link.href.as_str());
        }

        if let Some(url) = url {
            repo.remote("bitbucket", url)?;
        }

        Ok(())
    }
}
