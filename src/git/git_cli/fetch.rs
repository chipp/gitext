use crate::git::{exec_git_cmd, AuthDomainConfig};
use crate::Error;

use git2::{Remote, Repository};

pub fn fetch_remote<Conf>(
    remote: &mut Remote,
    repo: &Repository,
    _config: &Conf,
) -> Result<(), Error>
where
    Conf: AuthDomainConfig,
{
    let remote_name = remote.name().unwrap();
    println!("fetching remote {}", remote_name);

    exec_git_cmd(["fetch", remote_name], Some(repo))
}
