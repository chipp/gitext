use crate::git::find_remote_branch;
use crate::Error;

use git2::{
    build::CheckoutBuilder, Branch, BranchType, ErrorClass, ErrorCode, Oid, Remote, Repository,
};

pub fn switch_to_branch(
    branch_name: &str,
    commit: &str,
    remote: &Remote,
    repo: &Repository,
) -> Result<(), Error> {
    match find_remote_branch(branch_name, &remote, &repo) {
        Ok(remote_branch) => switch_to_existing_branch(branch_name, remote_branch, repo),
        Err(err) if err.class() == ErrorClass::Reference && err.code() == ErrorCode::NotFound => {
            // TODO: handle existing local branch
            let id = Oid::from_str(commit)?;
            let commit = repo.find_commit(id)?;

            let local_branch = repo.branch(branch_name, &commit, false)?;
            switch_to_local_branch(local_branch, &repo)
        }
        Err(err) => Err(err.into()),
    }
}

fn switch_to_existing_branch(
    branch_name: &str,
    remote_branch: Branch,
    repo: &Repository,
) -> Result<(), Error> {
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
        Err(err) => Err(err.into()),
    }
}

fn switch_to_local_branch(branch: Branch, repo: &Repository) -> Result<(), Error> {
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
