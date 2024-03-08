use git2::{Branch, BranchType, ErrorClass, ErrorCode, Repository};

use crate::error::Error;
use crate::exec_git_cmd;

pub fn switch_to_local_branch(branch: Branch, repo: &Repository) -> Result<(), Error> {
    let branch_name = branch.name().unwrap().unwrap();
    println!("switching to local branch {}", branch_name);

    exec_git_cmd(["switch", branch_name], Some(&repo))
}

pub fn switch_to_existing_branch(
    branch_name: &str,
    _remote_branch: Branch,
    repo: &Repository,
) -> Result<(), Error> {
    match repo.find_branch(branch_name, BranchType::Local) {
        Ok(local_branch) => switch_to_local_branch(local_branch, &repo),
        Err(err) if err.class() == ErrorClass::Reference && err.code() == ErrorCode::NotFound => {
            println!("switching to remote branch {}", branch_name);

            exec_git_cmd(["switch", "-c", branch_name], Some(&repo))
        }
        Err(err) => Err(err.into()),
    }
}
