use crate::Error;
use bitbucket::{
    get_bitbucket_remote, get_current_branch, get_current_repo_id, Client, PullRequest, RepoId,
};
use git2::{
    build::CheckoutBuilder, Branch, BranchType, Error as GitError, ErrorClass, ErrorCode, Oid,
    Remote, Repository,
};
use std::process::{Command, Stdio};
use std::str::FromStr;
use url::Url;

mod credential_helper;
use credential_helper::CredentialHelper;

pub struct Pr;

impl Pr {
    pub async fn handle(args: std::env::Args, repo: Repository) -> Result<(), Error> {
        let repo_id = get_current_repo_id(&repo).ok_or(Error::InvalidRepo)?;
        let branch = get_current_branch(&repo).ok_or(Error::Detached)?;

        let mut args = args;

        if let Some(arg) = args.next() {
            Self::handle_argument(arg, args.next(), repo_id, branch, repo).await
        } else {
            let existing_pr = Self::find_existing_pr(&branch, &repo_id).await?;

            let url = existing_pr
                .map(|pr| pr.url())
                .unwrap_or_else(|| Self::url_for_create(&branch, &repo_id));

            Self::open_url(url)
        }
    }

    async fn handle_argument(
        command: String,
        id: Option<String>,
        repo_id: RepoId,
        branch: String,
        repo: Repository,
    ) -> Result<(), Error> {
        match command.as_str() {
            "new" | "n" => Self::open_url(Self::url_for_create(&branch, &repo_id)),
            "info" | "i" => {
                if let Some(id) = id {
                    let id = u16::from_str(&id).map_err(|_| Error::InvalidPrId(command))?;

                    let client = Client::new();
                    let pr = client.get_pr_by_id(id, &repo_id).await?;

                    super::Prs::print_table_for_prs(&[pr]).await;

                    Ok(())
                } else {
                    let client = Client::new();
                    let mut prs = client
                        .find_prs_for_branch(&branch, &repo_id, "OPEN")
                        .await?;
                    prs.sort_unstable_by_key(|pr| std::cmp::Reverse(pr.id));

                    super::Prs::print_table_for_prs(&prs).await;

                    Ok(())
                }
            }
            "checkout" | "co" => {
                let id = id.ok_or(Error::InvalidPrId("empty".to_string()))?;
                let id = u16::from_str(&id).map_err(|_| Error::InvalidPrId(id))?;

                let client = Client::new();
                let pr = client
                    .get_pr_by_id(id, &repo_id)
                    .await
                    .map_err(|err| Error::NoPrWithId(id, err))?;

                Self::switch_to_branch(&pr, &repo)?;

                Ok(())
            }
            _ => Err(Error::UnknownSubCommand(command, &SUPPORTED_COMMANDS)),
        }
    }
}

const SUPPORTED_COMMANDS: [&str; 3] = ["new", "info", "checkout"];

impl Pr {
    async fn find_existing_pr(
        branch: &str,
        repo_id: &RepoId,
    ) -> Result<Option<PullRequest>, Error> {
        let client = Client::new();
        let prs = client.find_prs_for_branch(&branch, &repo_id, "ALL").await;

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

        match Self::find_remote_branch(branch_name, &remote, &repo) {
            Ok(remote_branch) => Self::switch_to_existing_branch(branch_name, remote_branch, repo),
            Err(err)
                if err.class() == ErrorClass::Reference && err.code() == ErrorCode::NotFound =>
            {
                // TODO: handle existing local branch
                let id = Oid::from_str(&pr.from_ref.latest_commit)?;
                let commit = repo.find_commit(id)?;

                let local_branch = repo.branch(&pr.from_ref.display_id, &commit, false)?;
                Self::switch_to_local_branch(local_branch, &repo)
            }
            Err(err) => Err(err),
        }
    }

    fn fetch_remote(remote: &mut Remote) -> Result<(), GitError> {
        use git2::RemoteCallbacks;

        println!("fetching remote {}", remote.name().unwrap());

        let mut credential_helper = CredentialHelper::new();

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(move |url, username_from_url, allowed_types| {
            credential_helper.credentials(url, username_from_url, allowed_types)
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

        println!("found remote branch {}", branch.name().unwrap().unwrap());

        Ok(branch)
    }

    fn switch_to_existing_branch(
        branch_name: &str,
        remote_branch: Branch,
        repo: &Repository,
    ) -> Result<(), GitError> {
        match repo.find_branch(branch_name, BranchType::Local) {
            Ok(local_branch) => Self::switch_to_local_branch(local_branch, &repo),
            Err(err)
                if err.class() == ErrorClass::Reference && err.code() == ErrorCode::NotFound =>
            {
                println!(
                    "creating a local branch from remote branch {}",
                    remote_branch.name().unwrap().unwrap()
                );

                let commit = remote_branch.get().peel_to_commit()?;

                let mut local_branch = repo.branch(branch_name, &commit, false)?;
                local_branch.set_upstream(remote_branch.name().unwrap())?;

                Self::switch_to_local_branch(local_branch, &repo)
            }
            Err(err) => Err(err),
        }
    }

    fn switch_to_local_branch(branch: Branch, repo: &Repository) -> Result<(), GitError> {
        println!(
            "switching to local branch {}",
            branch.name().unwrap().unwrap()
        );

        let reference = branch.get();
        let commit = reference.peel_to_commit()?;

        let mut checkout_builder = CheckoutBuilder::new();
        checkout_builder.safe();

        repo.checkout_tree(commit.as_object(), Some(&mut checkout_builder))?;
        repo.set_head(&reference.name().unwrap())?;

        Ok(())
    }
}
