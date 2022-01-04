use crate::common_git::{
    fetch_remote, find_remote_branch, get_current_branch, switch_to_existing_branch,
    switch_to_local_branch, AuthDomainConfig, BaseUrlConfig, JiraAuthDomainConfig, JiraUrlConfig,
};
use crate::gitlab::{get_current_repo_id, get_gitlab_remote, Client, PullRequest, RepoId};
use crate::Error;
use git2::{Error as GitError, ErrorClass, ErrorCode, Oid, Repository};
use std::process::{Command, Stdio};
use std::str::FromStr;
use url::Url;

pub struct Pr;

impl Pr {
    pub async fn handle<Conf>(
        args: std::env::Args,
        repo: Repository,
        config: Conf,
    ) -> Result<(), Error>
    where
        Conf: AuthDomainConfig + Send + Sync,
        Conf: BaseUrlConfig,
        Conf: JiraAuthDomainConfig + Send + Sync,
        Conf: JiraUrlConfig,
    {
        let repo_id = get_current_repo_id(&repo, &config).ok_or(Error::InvalidRepo)?;
        let branch = get_current_branch(&repo).ok_or(Error::Detached)?;

        let mut args = args;

        if let Some(arg) = args.next() {
            Self::handle_argument(arg, args.next(), repo_id, &branch, repo, &config).await
        } else {
            let existing_pr = Self::find_existing_pr(&branch, &repo_id, &config).await?;

            let url = existing_pr
                .map(|pr| pr.url)
                .unwrap_or_else(|| Self::url_for_create(&branch, &repo_id, None, &config));

            Self::open_url(url)
        }
    }

    async fn handle_argument<Conf>(
        command: String,
        id: Option<String>,
        repo_id: RepoId,
        branch: &str,
        repo: Repository,
        config: &Conf,
    ) -> Result<(), Error>
    where
        Conf: AuthDomainConfig + Send + Sync,
        Conf: BaseUrlConfig,
        Conf: JiraAuthDomainConfig + Send + Sync,
        Conf: JiraUrlConfig,
    {
        match command.as_str() {
            "new" | "n" => Self::open_url(Self::url_for_create(branch, &repo_id, id, config)),
            "browse" | "b" => {
                if let Some(id) = id {
                    let mut url = repo_id.url(&config.base_url());

                    {
                        let mut segments = url.path_segments_mut().unwrap();

                        let _ = u16::from_str(&id).map_err(|_| Error::InvalidPrId(id.clone()))?;

                        segments.push("-");
                        segments.push("merge_requests");
                        segments.push(&id);
                    }

                    Self::open_url(url)
                } else {
                    Err(Error::InvalidPrId(String::new()))
                }
            }
            "info" | "i" => {
                if let Some(id) = id {
                    let id = u16::from_str(&id).map_err(|_| Error::InvalidPrId(id))?;

                    let client = Client::new(config);
                    let pr = client.get_pr_by_id(id, &repo_id).await?;

                    super::prs::Prs::print_table_for_prs(&[pr], config).await;

                    Ok(())
                } else {
                    let client = Client::new(config);
                    let mut prs = client
                        .find_prs_for_branch(&branch, &repo_id, "open")
                        .await?;
                    prs.sort_unstable_by_key(|pr| std::cmp::Reverse(pr.id));

                    super::prs::Prs::print_table_for_prs(&prs, config).await;

                    Ok(())
                }
            }
            "checkout" | "co" => {
                let id = id.ok_or(Error::InvalidPrId("empty".to_string()))?;
                let id = u16::from_str(&id).map_err(|_| Error::InvalidPrId(id))?;

                let client = Client::new(config);
                let pr = client
                    .get_pr_by_id(id, &repo_id)
                    .await
                    .map_err(|err| Error::NoPrWithId(id, err))?;

                Self::switch_to_branch(&pr, &repo, config)?;

                Ok(())
            }
            _ => Err(Error::UnknownSubCommand(command, &SUPPORTED_COMMANDS)),
        }
    }
}

const SUPPORTED_COMMANDS: [&str; 3] = ["new", "info", "checkout"];

impl Pr {
    async fn find_existing_pr<Conf>(
        branch: &str,
        repo_id: &RepoId,
        config: &Conf,
    ) -> Result<Option<PullRequest>, Error>
    where
        Conf: BaseUrlConfig,
        Conf: AuthDomainConfig + Send + Sync,
    {
        let client = Client::new(config);
        let prs = client.find_prs_for_branch(&branch, &repo_id, "all").await;

        let mut prs = prs.map_err(|err| Error::NoPrsForBranch(branch.to_string(), err))?;
        prs.sort_unstable_by(|lhs, rhs| lhs.state.cmp(&rhs.state));

        Ok(prs.into_iter().next())
    }

    fn url_for_create<Conf>(
        branch: &str,
        repo_id: &RepoId,
        target: Option<String>,
        config: &Conf,
    ) -> Url
    where
        Conf: BaseUrlConfig,
    {
        let mut url = repo_id.url(&config.base_url());

        {
            let mut segments = url.path_segments_mut().unwrap();
            segments.push("-");
            segments.push("merge_requests");
            segments.push("new");
        }

        {
            let mut query_pairs = url.query_pairs_mut();
            query_pairs.append_pair("merge_request[source_branch]", &branch);

            if let Some(target) = target {
                query_pairs.append_pair("merge_request[target_branch]", &target);
            }
        }

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
        Conf: BaseUrlConfig,
        Conf: AuthDomainConfig,
    {
        let branch_name: &str = &pr.source_branch;
        let mut remote = get_gitlab_remote(&repo, config).unwrap();
        fetch_remote(&mut remote, config)?;

        match find_remote_branch(branch_name, &remote, &repo) {
            Ok(remote_branch) => switch_to_existing_branch(branch_name, remote_branch, repo),
            Err(err)
                if err.class() == ErrorClass::Reference && err.code() == ErrorCode::NotFound =>
            {
                // TODO: handle existing local branch
                let id = Oid::from_str(&pr.sha)?;
                let commit = repo.find_commit(id)?;

                let local_branch = repo.branch(&pr.source_branch, &commit, false)?;
                switch_to_local_branch(local_branch, &repo)
            }
            Err(err) => Err(err),
        }
    }
}
