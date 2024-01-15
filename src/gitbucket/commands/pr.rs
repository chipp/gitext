use std::process::{Command, Stdio};
use std::str::FromStr;

use crate::bitbucket::{get_bitbucket_remote, get_current_repo_id, Client, PullRequest, RepoId};
use crate::common_git::{
    fetch_remote, find_remote_branch, get_current_branch, switch_to_existing_branch,
    switch_to_local_branch, AuthDomainConfig, BaseUrlConfig, JiraAuthDomainConfig, JiraUrlConfig,
};
use crate::error::Error;

use git2::{Error as GitError, ErrorClass, ErrorCode, Oid, Repository};
use url::Url;

pub struct Pr;

impl Pr {
    pub async fn handle<Arg: AsRef<str>, Conf>(
        args: &[Arg],
        repo: &Repository,
        config: &Conf,
    ) -> Result<(), Error>
    where
        Conf: AuthDomainConfig + Send + Sync,
        Conf: BaseUrlConfig,
        Conf: JiraAuthDomainConfig + Send + Sync,
        Conf: JiraUrlConfig,
    {
        let repo_id = get_current_repo_id(&repo, config).ok_or(Error::InvalidRepo)?;
        let branch = get_current_branch(&repo).ok_or(Error::Detached)?;

        if let Some(arg) = args.get(0) {
            Self::handle_argument(arg, args.get(1), repo_id, branch, repo, config).await
        } else {
            let existing_pr = Self::find_existing_open_pr(&branch, &repo_id, config).await?;

            let url = existing_pr
                .map(|pr| pr.url(&config.base_url()))
                .unwrap_or_else(|| Self::url_for_create(&branch, &repo_id, config));

            Self::open_url(url)
        }
    }

    async fn handle_argument<Arg: AsRef<str>, Conf>(
        command: Arg,
        id: Option<Arg>,
        repo_id: RepoId,
        branch: String,
        repo: &Repository,
        config: &Conf,
    ) -> Result<(), Error>
    where
        Conf: AuthDomainConfig + Send + Sync,
        Conf: BaseUrlConfig,
        Conf: JiraAuthDomainConfig + Send + Sync,
        Conf: JiraUrlConfig,
    {
        match command.as_ref() {
            "new" | "n" => Self::open_url(Self::url_for_create(&branch, &repo_id, config)),
            "browse" | "b" => {
                if let Some(id) = id {
                    let mut url = repo_id.url(config.base_url());

                    {
                        let mut segments = url.path_segments_mut().unwrap();

                        let id = u16::from_str(id.as_ref())
                            .map_err(|_| Error::InvalidPrId(id.as_ref().to_string()))?;

                        segments.push("pull-requests");
                        segments.push(&format!("{}", id));
                        segments.push("overview");
                    }

                    Self::open_url(url)
                } else {
                    Err(Error::InvalidPrId(String::new()))
                }
            }
            "info" | "i" => {
                if let Some(id) = id {
                    let id = u16::from_str(id.as_ref())
                        .map_err(|_| Error::InvalidPrId(id.as_ref().to_string()))?;

                    let client = Client::new(config);
                    let pr = client.get_pr_by_id(id, &repo_id).await?;

                    super::prs::Prs::print_table_for_prs(&[pr], config).await;

                    Ok(())
                } else {
                    let client = Client::new(config);
                    let mut prs = client
                        .find_prs_for_branch(&branch, &repo_id, "OPEN")
                        .await?;
                    prs.sort_unstable_by_key(|pr| std::cmp::Reverse(pr.id));

                    super::prs::Prs::print_table_for_prs(&prs, config).await;

                    Ok(())
                }
            }
            "checkout" | "co" => {
                let id = id.ok_or(Error::InvalidPrId("empty".to_string()))?;
                let id = u16::from_str(id.as_ref())
                    .map_err(|_| Error::InvalidPrId(id.as_ref().to_string()))?;

                let client = Client::new(config);
                let pr = client
                    .get_pr_by_id(id, &repo_id)
                    .await
                    .map_err(|err| Error::NoPrWithId(id, err))?;

                Self::switch_to_branch(&pr, &repo, config)?;

                Ok(())
            }
            _ => Err(Error::UnknownSubCommand(
                command.as_ref().to_string(),
                &SUPPORTED_COMMANDS,
            )),
        }
    }
}

const SUPPORTED_COMMANDS: [&str; 4] = ["new", "browse", "info", "checkout"];

impl Pr {
    async fn find_existing_open_pr<Conf>(
        branch: &str,
        repo_id: &RepoId,
        config: &Conf,
    ) -> Result<Option<PullRequest>, Error>
    where
        Conf: AuthDomainConfig + Send + Sync,
        Conf: BaseUrlConfig,
    {
        let client = Client::new(config);
        let prs = client.find_prs_for_branch(&branch, &repo_id, "OPEN").await;

        let prs = prs.map_err(|err| Error::NoPrsForBranch(branch.to_string(), err))?;
        Ok(prs.into_iter().next())
    }

    fn url_for_create<Conf>(branch: &str, repo_id: &RepoId, config: &Conf) -> Url
    where
        Conf: BaseUrlConfig,
    {
        let mut url = repo_id.url(&config.base_url());

        {
            let mut segments = url.path_segments_mut().unwrap();
            segments.push("pull-requests");
        }

        url.query_pairs_mut()
            .append_pair("at", &branch)
            .append_pair("create", "")
            .append_pair("sourceBranch", &branch);

        url
    }

    fn open_url(url: Url) -> Result<(), Error> {
        Command::new("open")
            .arg(url.as_str())
            .stdout(Stdio::null())
            .spawn()
            .map(|_| ())
            .map_err(|err| Error::OpenUrl(err, url))
    }
}

impl Pr {
    fn switch_to_branch<Conf>(
        pr: &PullRequest,
        repo: &Repository,
        config: &Conf,
    ) -> Result<(), GitError>
    where
        Conf: AuthDomainConfig,
        Conf: BaseUrlConfig,
    {
        let branch_name: &str = &pr.from_ref.display_id;
        let mut remote = get_bitbucket_remote(&repo, config).unwrap();
        fetch_remote(&mut remote, config)?;

        match find_remote_branch(branch_name, &remote, &repo) {
            Ok(remote_branch) => switch_to_existing_branch(branch_name, remote_branch, repo),
            Err(err)
                if err.class() == ErrorClass::Reference && err.code() == ErrorCode::NotFound =>
            {
                // TODO: handle existing local branch
                let id = Oid::from_str(&pr.from_ref.latest_commit)?;
                let commit = repo.find_commit(id)?;

                let local_branch = repo.branch(&pr.from_ref.display_id, &commit, false)?;
                switch_to_local_branch(local_branch, &repo)
            }
            Err(err) => Err(err),
        }
    }
}
