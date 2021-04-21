use bitbucket::{get_current_branch, get_current_repo_id, Client, PullRequest, RepoId};
use git2::Repository;
use std::process::{Command, Stdio};
use url::Url;

pub struct Pr;

impl Pr {
    pub async fn handle(_args: std::env::Args, repo: Repository) -> Result<(), String> {
        let repo_id =
            get_current_repo_id(&repo).ok_or(String::from("this is not a bitbucket repository"))?;
        let branch =
            get_current_branch(&repo).ok_or(String::from("can't find the current branch"))?;

        let existing_pr = Self::find_existing_pr(&branch, &repo_id).await?;

        let url = existing_pr
            .map(|pr| Self::url_for_existing_pr(&repo_id, &pr))
            .unwrap_or_else(|| Self::url_for_create(&branch, &repo_id));

        Command::new("open")
            .arg(url.as_str())
            .stdout(Stdio::null())
            .spawn()
            .map(|_| ())
            .map_err(|e| format!("can't open URL {}: {}", url, e))
    }

    async fn find_existing_pr(
        branch: &str,
        repo_id: &RepoId,
    ) -> Result<Option<PullRequest>, String> {
        let client = Client::new();
        let prs = client.find_prs_for_branch(&branch, &repo_id).await;
        let prs = prs.map_err(|err| format!("can't find prs for branch {}: {}", branch, err))?;

        Ok(prs.into_iter().next())
    }

    fn url_for_existing_pr(repo_id: &RepoId, pr: &PullRequest) -> Url {
        let mut url = repo_id.url();

        {
            let mut segments = url.path_segments_mut().unwrap();
            segments.push("pull-requests");
            segments.push(&format!("{}", pr.id));
        }

        url
    }

    fn url_for_create(branch: &str, repo_id: &RepoId) -> Url {
        let mut url = repo_id.url();

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
}
