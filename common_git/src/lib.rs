mod config;

pub use config::{get_config, Config, GetConfigError};
pub use config::{AuthDomainConfig, BaseUrlConfig, JiraUrlConfig};

use git2::{Error as GitError, Repository, RepositoryOpenFlags};

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
