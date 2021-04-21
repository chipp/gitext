use crate::Error;
use bitbucket::{
    get_bitbucket_remote, get_current_branch, get_current_repo_id, Client, PullRequest, RepoId,
};
use git2::{
    build::CheckoutBuilder, Branch, BranchType, Error as GitError, ErrorClass, ErrorCode, Remote,
    Repository,
};
use std::process::{Command, Stdio};
use std::str::FromStr;
use url::Url;

pub struct Pr;

impl Pr {
    pub async fn handle(args: std::env::Args, repo: Repository) -> Result<(), Error> {
        let repo_id = get_current_repo_id(&repo).ok_or(Error::InvalidRepo)?;
        let branch = get_current_branch(&repo).ok_or(Error::Detached)?;

        let mut args = args;

        if let Some(pr) = args.next() {
            if &pr == "new" {
                Self::open_url(Self::url_for_create(&branch, &repo_id))
            } else {
                let id = u16::from_str(&pr).map_err(|_| Error::InvalidPrId(pr))?;

                let client = Client::new();
                let pr = client
                    .get_pr_by_id(id, &repo_id)
                    .await
                    .map_err(|err| Error::NoPrWithId(id, err))?;

                Self::switch_to_branch(&pr, &repo)?;

                Ok(())
            }
        } else {
            let existing_pr = Self::find_existing_pr(&branch, &repo_id).await?;

            let url = existing_pr
                .map(|pr| pr.url())
                .unwrap_or_else(|| Self::url_for_create(&branch, &repo_id));

            Self::open_url(url)
        }
    }
}

impl Pr {
    async fn find_existing_pr(
        branch: &str,
        repo_id: &RepoId,
    ) -> Result<Option<PullRequest>, Error> {
        let client = Client::new();
        let prs = client.find_prs_for_branch(&branch, &repo_id).await;

        let mut prs = prs.map_err(|err| Error::NoPrsForBranch(branch.to_string(), err))?;
        prs.sort_unstable_by(|lhs, rhs| lhs.state.cmp(&rhs.state));

        Ok(prs.into_iter().next())
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
    fn switch_to_branch(pr: &PullRequest, repo: &Repository) -> Result<(), GitError> {
        let branch_name: &str = &pr.from_ref.display_id;
        let mut remote = get_bitbucket_remote(&repo).unwrap();
        Self::fetch_remote(&mut remote)?;

        // TODO: switch to commit
        let remote_branch = Self::find_remote_branch(branch_name, &remote, &repo)?;

        match repo.find_branch(branch_name, BranchType::Local) {
            Ok(local_branch) => Self::switch_to_local_branch(local_branch, &repo),
            Err(err)
                if err.class() == ErrorClass::Reference && err.code() == ErrorCode::NotFound =>
            {
                let commit = remote_branch.get().peel_to_commit()?;

                let mut local_branch = repo.branch(branch_name, &commit, false)?;
                local_branch.set_upstream(remote_branch.name().unwrap())?;

                Self::switch_to_local_branch(local_branch, &repo)
            }
            Err(err) => Err(err),
        }
    }

    fn fetch_remote(remote: &mut Remote) -> Result<(), GitError> {
        use git2::{Cred, RemoteCallbacks};

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_, username_from_url, _| {
            Cred::ssh_key_from_agent(username_from_url.unwrap())
        });

        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(callbacks);

        remote.fetch(&[], Some(&mut fo), None)?;
        Ok(())
    }

    fn find_remote_branch<'repo>(
        branch_name: &str,
        remote: &Remote,
        repo: &'repo Repository,
    ) -> Result<Branch<'repo>, GitError> {
        let remote_name = remote.name().unwrap();
        let branch_name = format!("{}/{}", remote_name, branch_name);

        let branch = repo.find_branch(&branch_name, BranchType::Remote)?;
        Ok(branch)
    }

    fn switch_to_local_branch(branch: Branch, repo: &Repository) -> Result<(), GitError> {
        let reference = branch.get();
        let commit = reference.peel_to_commit()?;

        let mut checkout_builder = CheckoutBuilder::new();
        checkout_builder.safe();

        repo.checkout_tree(commit.as_object(), Some(&mut checkout_builder))?;
        repo.set_head(&reference.name().unwrap())?;

        Ok(())
    }
}
