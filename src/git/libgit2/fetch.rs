use git2::{Remote, RemoteCallbacks, Repository};

use super::credential_helper::CredentialHelper;
use crate::{error::Error, git::AuthDomainConfig};

pub fn fetch_remote<Conf>(
    remote: &mut Remote,
    _repo: &Repository,
    config: &Conf,
) -> Result<(), Error>
where
    Conf: AuthDomainConfig,
{
    println!("fetching remote {}", remote.name().unwrap());

    let mut credential_helper = CredentialHelper::new();

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |url, username_from_url, allowed_types| {
        credential_helper.credentials(url, username_from_url, allowed_types, config)
    });

    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(callbacks);

    remote.fetch::<&str>(&[], Some(&mut fo), None)?;
    Ok(())
}
