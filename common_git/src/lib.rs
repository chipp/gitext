mod config;
mod credential_helper;

pub use config::{get_config, Config, GetConfigError, Provider};
pub use config::{AuthDomainConfig, BaseUrlConfig, JiraUrlConfig};
pub use credential_helper::CredentialHelper;
use git2::build::CheckoutBuilder;
pub use git2::{Error as GitError, Repository};

use git2::{Branch, BranchType, ErrorClass, ErrorCode, Remote, RepositoryOpenFlags};
use regex::Regex;

pub fn get_repo(path: &str) -> Result<Repository, GitError> {
    Repository::open_ext(
        path,
        RepositoryOpenFlags::empty(),
        vec![dirs::home_dir().unwrap()],
    )
}

pub fn get_current_branch(repo: &Repository) -> Option<String> {
    let head = repo.head().ok()?;

    if head.is_branch() {
        head.name()
            .map(|name| name.trim_start_matches("refs/heads/"))
            .map(String::from)
    } else {
        None
    }
}

pub fn find_remote_branch<'repo>(
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

pub fn switch_to_existing_branch(
    branch_name: &str,
    remote_branch: Branch,
    repo: &Repository,
) -> Result<(), GitError> {
    match repo.find_branch(branch_name, BranchType::Local) {
        Ok(local_branch) => switch_to_local_branch(local_branch, &repo),
        Err(err) if err.class() == ErrorClass::Reference && err.code() == ErrorCode::NotFound => {
            println!(
                "creating a local branch from remote branch {}",
                remote_branch.name().unwrap().unwrap()
            );

            let commit = remote_branch.get().peel_to_commit()?;

            let mut local_branch = repo.branch(branch_name, &commit, false)?;
            local_branch.set_upstream(remote_branch.name().unwrap())?;

            switch_to_local_branch(local_branch, &repo)
        }
        Err(err) => Err(err),
    }
}

pub fn switch_to_local_branch(branch: Branch, repo: &Repository) -> Result<(), GitError> {
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

pub fn fetch_remote<Conf>(remote: &mut Remote, config: &Conf) -> Result<(), GitError>
where
    Conf: AuthDomainConfig,
{
    use git2::RemoteCallbacks;

    println!("fetching remote {}", remote.name().unwrap());

    let mut credential_helper = CredentialHelper::new();

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |url, username_from_url, allowed_types| {
        credential_helper.credentials(url, username_from_url, allowed_types, config)
    });

    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(callbacks);

    remote.fetch(&[], Some(&mut fo), None)?;
    Ok(())
}

pub fn extract_ticket<'b>(branch: &'b str) -> Option<&'b str> {
    let re = Regex::new(r"\w{2,}-\d+").unwrap();
    re.captures(&branch)
        .map(|caps| caps.get(0).unwrap().as_str())
}
