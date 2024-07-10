use std::process::{Command, Stdio};

use crate::git::{
    fetch_remote, get_current_branch, switch_to_branch, AuthDomainConfig, BaseUrlConfig,
    JiraUrlConfig,
};
use crate::gitlab::{get_current_repo_id, get_gitlab_remote, Client, PullRequest, RepoId};
use crate::Error;

use clap::ArgMatches;
use git2::Repository;
use url::Url;

pub struct Pr;

impl Pr {
    pub async fn handle<Conf>(
        args: &ArgMatches,
        repo: &Repository,
        config: &Conf,
    ) -> Result<(), Error>
    where
        Conf: AuthDomainConfig + Send + Sync,
        Conf: BaseUrlConfig,
        Conf: JiraUrlConfig,
    {
        let repo_id = get_current_repo_id(&repo, config).ok_or(Error::InvalidRepo)?;
        let branch = get_current_branch(&repo).ok_or(Error::Detached)?;

        let command = args.subcommand().unwrap_or(("new-or-browse", args));

        match command {
            ("browse", args) => {
                let id: u16 = *args.get_one("id").expect("required");

                let mut url = repo_id.url(&config.base_url());

                {
                    let mut segments = url.path_segments_mut().unwrap();

                    segments.push("-");
                    segments.push("merge_requests");
                    segments.push(&format!("{}", id));
                }

                Self::open_url(url)
            }
            ("checkout", args) => {
                let id: u16 = *args.get_one("id").expect("required");

                let client = Client::new(config);
                let pr = client
                    .get_pr_by_id(id, &repo_id)
                    .await
                    .map_err(|err| Error::NoPrWithId(id, err))?;

                Self::switch(&pr, &repo, config)?;

                Ok(())
            }
            ("info", args) => {
                if let Some(id) = args.get_one::<u16>("id") {
                    let client = Client::new(config);
                    let pr = client.get_pr_by_id(*id, &repo_id).await?;

                    super::prs::Prs::print_table_for_prs(&[pr], &repo_id, config).await;

                    Ok(())
                } else {
                    let client = Client::new(config);
                    let mut prs = client
                        .find_prs_for_branch(&branch, &repo_id, "open")
                        .await?;
                    prs.sort_unstable_by_key(|pr| std::cmp::Reverse(pr.id));

                    if prs.is_empty() {
                        println!("No PRs for that branch");
                        return Ok(());
                    }

                    super::prs::Prs::print_table_for_prs(&prs, &repo_id, config).await;

                    Ok(())
                }
            }
            ("new", args) => Self::open_url(Self::url_for_create(
                &branch,
                &repo_id,
                args.get_one::<String>("target").map(String::as_str),
                config,
            )),
            ("new-or-browse", _) => {
                let existing_pr = Self::find_existing_open_pr(&branch, &repo_id, config).await?;

                let url = existing_pr
                    .map(|pr| pr.url)
                    .unwrap_or_else(|| Self::url_for_create(&branch, &repo_id, None, config));

                Self::open_url(url)
            }
            _ => unreachable!(),
        }
    }
}

impl Pr {
    async fn find_existing_open_pr<Conf>(
        branch: &str,
        repo_id: &RepoId,
        config: &Conf,
    ) -> Result<Option<PullRequest>, Error>
    where
        Conf: BaseUrlConfig,
        Conf: AuthDomainConfig + Send + Sync,
    {
        let client = Client::new(config);
        let prs = client
            .find_prs_for_branch(&branch, &repo_id, "opened")
            .await;

        let prs = prs.map_err(|err| Error::NoPrsForBranch(branch.to_string(), err))?;
        Ok(prs.into_iter().next())
    }

    fn url_for_create<Conf>(
        branch: &str,
        repo_id: &RepoId,
        target: Option<&str>,
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
                query_pairs.append_pair("merge_request[target_branch]", target.as_ref());
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
    fn switch<Conf>(pr: &PullRequest, repo: &Repository, config: &Conf) -> Result<(), Error>
    where
        Conf: BaseUrlConfig,
        Conf: AuthDomainConfig,
    {
        let branch_name = &pr.source_branch;
        let commit_sha = &pr.sha;

        let mut remote = get_gitlab_remote(&repo, config).unwrap();
        fetch_remote(&mut remote, repo, config)?;

        switch_to_branch(branch_name, commit_sha, &remote, repo)
    }
}
