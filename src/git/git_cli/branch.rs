use git2::{Branch, BranchType, ErrorClass, ErrorCode, Remote, Repository};

use crate::error::Error;
use crate::exec_git_cmd;
use crate::git::find_remote_branch;

pub fn switch_to_branch(
    branch_name: &str,
    commit: &str,
    remote: &Remote,
    repo: &Repository,
) -> Result<(), Error> {
    match find_remote_branch(branch_name, &remote, &repo) {
        Ok(remote_branch) => switch_to_existing_branch(remote_branch, branch_name, repo),
        Err(err) if err.class() == ErrorClass::Reference && err.code() == ErrorCode::NotFound => {
            exec_git_cmd(["branch", branch_name, commit], Some(repo))?;
            switch_to_local_branch(branch_name, &repo)
        }
        Err(err) => Err(err.into()),
    }
}

fn switch_to_local_branch(branch_name: &str, repo: &Repository) -> Result<(), Error> {
    println!("switching to local branch {}", branch_name);

    exec_git_cmd(["switch", branch_name], Some(&repo))
}

fn switch_to_existing_branch(
    remote_branch: Branch,
    branch_name: &str,
    repo: &Repository,
) -> Result<(), Error> {
    let remote_branch_name = remote_branch.name()?.unwrap();

    match repo.find_branch(branch_name, BranchType::Local) {
        Ok(_) => switch_to_local_branch(branch_name, &repo),
        Err(err) if err.class() == ErrorClass::Reference && err.code() == ErrorCode::NotFound => {
            println!("switching to remote branch {}", branch_name);

            exec_git_cmd(["branch", branch_name, remote_branch_name], Some(repo))?;
            exec_git_cmd(["switch", branch_name], Some(&repo))
        }
        Err(err) => Err(err.into()),
    }
}
