use std::str::FromStr;

use crate::git::{fetch_remote, switch_to_branch, AuthDomainConfig, BaseUrlConfig};
use crate::github::{get_current_repo_id, get_github_remote, Client, PullRequest};
use crate::Error;

use clap::ArgMatches;
use git2::Repository;

pub struct Switch;

impl Switch {
    pub async fn handle<Conf>(
        args: &ArgMatches,
        repo: &Repository,
        config: &Conf,
    ) -> Result<bool, Error>
    where
        Conf: AuthDomainConfig + Send + Sync,
        Conf: BaseUrlConfig,
    {
        let repo_id = get_current_repo_id(&repo, config).ok_or(Error::InvalidRepo)?;

        let id: &str = args.get_one::<String>("id").expect("required");
        let id = match u16::from_str(id.as_ref()) {
            Ok(id) => id,
            Err(_) => return Ok(false),
        };

        let client = Client::new(config);
        let pr = client
            .get_pr_by_id(id, &repo_id)
            .await
            .map_err(|err| Error::NoPrWithId(id, err))?;

        Self::switch(&pr, &repo, config)?;

        Ok(true)
    }

    fn switch<Conf>(pr: &PullRequest, repo: &Repository, config: &Conf) -> Result<(), Error>
    where
        Conf: BaseUrlConfig,
        Conf: AuthDomainConfig,
    {
        let branch_name = &pr.head.reference;
        let commit_sha = &pr.head.sha;

        let mut remote = get_github_remote(&repo, config).unwrap();
        fetch_remote(&mut remote, repo, config)?;

        switch_to_branch(branch_name, commit_sha, &remote, repo)
    }
}
