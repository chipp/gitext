use std::path::Path;

use git2::Repository;

use crate::error::Error;
use crate::git::{exec_git_cmd, get_repo, AuthDomainConfig};

pub fn clone_repo<Conf>(url: &str, path: &Path, _config: &Conf) -> Result<Repository, Error>
where
    Conf: AuthDomainConfig,
{
    let args = ["clone", url, path.to_str().unwrap()];
    exec_git_cmd(&args, None)?;

    let repo = get_repo(path)?;
    Ok(repo)
}
