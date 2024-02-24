use std::str::FromStr;

use crate::common_git::{
    fetch_remote, find_remote_branch, switch_to_existing_branch, switch_to_local_branch,
    AuthDomainConfig, BaseUrlConfig,
};
use crate::github::{get_current_repo_id, get_github_remote, Client, PullRequest};
use crate::Error;

use clap::ArgMatches;
use git2::{Error as GitError, ErrorClass, ErrorCode, Oid, Repository};

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

        Self::switch_to_branch(&pr, &repo, config)?;

        Ok(true)
    }

    fn switch_to_branch<Conf>(
        pr: &PullRequest,
        repo: &Repository,
        config: &Conf,
    ) -> Result<(), GitError>
    where
        Conf: BaseUrlConfig,
        Conf: AuthDomainConfig,
    {
        let branch_name: &str = &pr.head.reference;
        let mut remote = get_github_remote(&repo, config).unwrap();
        fetch_remote(&mut remote, config)?;

        match find_remote_branch(branch_name, &remote, &repo) {
            Ok(remote_branch) => switch_to_existing_branch(branch_name, remote_branch, repo),
            Err(err)
                if err.class() == ErrorClass::Reference && err.code() == ErrorCode::NotFound =>
            {
                // TODO: handle existing local branch
                let id = Oid::from_str(&pr.head.sha)?;
                let commit = repo.find_commit(id)?;

                let local_branch = repo.branch(&pr.head.reference, &commit, false)?;
                switch_to_local_branch(local_branch, &repo)
            }
            Err(err) => Err(err),
        }
    }
}
