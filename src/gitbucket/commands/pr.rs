use std::collections::HashMap;
use std::process::{Command, Stdio};

use crate::bitbucket::{get_bitbucket_remote, get_current_repo_id, Client, PullRequest, RepoId};
use crate::error::Error;
use crate::git::{
    fetch_remote, get_current_branch, switch_to_branch, AuthDomainConfig, BaseUrlConfig,
    JiraAuthDomainConfig, JiraUrlConfig,
};

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
        Conf: JiraAuthDomainConfig + Send + Sync,
        Conf: JiraUrlConfig,
    {
        let repo_id = get_current_repo_id(&repo, config).ok_or(Error::InvalidRepo)?;
        let branch = get_current_branch(&repo).ok_or(Error::Detached)?;

        let command = args.subcommand().unwrap_or(("new-or-browse", args));

        match command {
            ("browse", args) => {
                let id: u16 = *args.get_one("id").expect("required");

                let mut url = repo_id.url(config.base_url());

                {
                    let mut segments = url.path_segments_mut().unwrap();

                    segments.push("pull-requests");
                    segments.push(&format!("{}", id));
                    segments.push("overview");
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
                let client = Client::new(config);

                if let Some(id) = args.get_one::<u16>("id") {
                    let pr = client.get_pr_by_id(*id, &repo_id).await?;
                    let build_stats = client
                        .get_commits_build_stats(&[&pr.from_ref.latest_commit])
                        .await?
                        .into_iter()
                        .map(|(k, v)| (k, v.into()))
                        .collect::<HashMap<_, _>>();

                    super::prs::Prs::print_table_for_prs(&[pr], build_stats, config).await;

                    Ok(())
                } else {
                    let mut prs = client
                        .find_prs_for_branch(&branch, &repo_id, "OPEN")
                        .await?;
                    prs.sort_unstable_by_key(|pr| std::cmp::Reverse(pr.id));

                    let shas = prs
                        .iter()
                        .map(|pr| pr.from_ref.latest_commit.as_str())
                        .collect::<Vec<_>>();

                    let build_stats = client
                        .get_commits_build_stats(&shas)
                        .await?
                        .into_iter()
                        .map(|(k, v)| (k, v.into()))
                        .collect::<HashMap<_, _>>();

                    super::prs::Prs::print_table_for_prs(&prs, build_stats, config).await;

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
                    .map(|pr| pr.url(&config.base_url()))
                    .unwrap_or_else(|| Self::url_for_create(&branch, &repo_id, None, config));

                Self::open_url(url)
            }
            (&_, _) => todo!(),
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
        Conf: AuthDomainConfig + Send + Sync,
        Conf: BaseUrlConfig,
    {
        let client = Client::new(config);
        let prs = client.find_prs_for_branch(&branch, &repo_id, "OPEN").await;

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
            segments.push("pull-requests");
        }

        {
            let mut pairs = url.query_pairs_mut();

            pairs
                .append_pair("create", "")
                .append_pair("sourceBranch", branch);

            if let Some(target) = target {
                pairs.append_pair("targetBranch", target);
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
        let branch_name = &pr.from_ref.display_id;
        let commit_sha = &pr.from_ref.latest_commit;

        let mut remote = get_bitbucket_remote(&repo, config).unwrap();
        fetch_remote(&mut remote, repo, config)?;

        switch_to_branch(branch_name, commit_sha, &remote, repo)
    }
}
