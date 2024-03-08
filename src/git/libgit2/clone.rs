mod formatter;
mod reporter;

use std::cell::RefCell;
use std::io::{self, Write};
use std::path::Path;

use super::credential_helper::CredentialHelper;
use crate::git::AuthDomainConfig;
use crate::Error;
use reporter::{report_clone, State};

use git2::build::RepoBuilder;
use git2::Repository;

pub fn clone_repo<Conf>(url: &str, path: &Path, config: &Conf) -> Result<Repository, Error>
where
    Conf: AuthDomainConfig,
{
    let mut credential_helper = CredentialHelper::new();

    let state = RefCell::new(State::new());

    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(move |url, username_from_url, allowed_types| {
        credential_helper.credentials(url, username_from_url, allowed_types, config)
    });
    callbacks.transfer_progress(|stats| {
        let mut state = state.borrow_mut();
        report_clone(&mut *state, stats.to_owned());
        true
    });
    callbacks.sideband_progress(|text| {
        if let Ok(text) = std::str::from_utf8(text) {
            eprint!("remote: {text}");
            io::stderr().flush().unwrap();
        }
        true
    });

    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(callbacks);

    let repo = RepoBuilder::new().fetch_options(fo).clone(url, path)?;
    Ok(repo)
}
