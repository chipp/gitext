mod config;

#[cfg(feature = "git-cli")]
mod git_cli {
    pub mod branch;
    pub mod clone;
    pub mod fetch;
}

#[cfg(feature = "git-cli")]
pub use git_cli::branch::switch_to_branch;

#[cfg(feature = "git-cli")]
pub use git_cli::clone::clone_repo;

#[cfg(feature = "git-cli")]
pub use git_cli::fetch::fetch_remote;

#[cfg(not(feature = "git-cli"))]
mod libgit2 {
    pub mod branch;
    pub mod clone;
    pub mod fetch;

    mod credential_helper;
}

#[cfg(not(feature = "git-cli"))]
pub use libgit2::branch::switch_to_branch;

#[cfg(not(feature = "git-cli"))]
pub use libgit2::clone::clone_repo;

#[cfg(not(feature = "git-cli"))]
pub use libgit2::fetch::fetch_remote;

use std::ffi::OsStr;
use std::path::Path;
use std::process::{exit, Command};

pub use config::{
    get_aliases_from_config, get_config, set_config, set_provider, Config, ConfigError, Provider,
};
pub use config::{AuthDomainConfig, BaseUrlConfig, JiraUrlConfig};

use git2::{Branch, BranchType, Remote, RepositoryOpenFlags};
use git2::{Error as GitError, Repository};
use regex::Regex;

use super::Error;

pub fn get_repo(path: &Path) -> Result<Repository, GitError> {
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

pub fn get_current_branch_upstream_remote(repo: &Repository) -> Option<String> {
    let branch = get_current_branch(repo)?;
    let refname = format!("refs/heads/{branch}");

    repo.branch_upstream_remote(&refname)
        .ok()?
        .as_str()
        .map(String::from)
}

pub fn find_remote_by_priority<'repo, T, F>(repo: &'repo Repository, mut f: F) -> Option<T>
where
    F: FnMut(Remote<'repo>) -> Option<T>,
{
    let remotes = repo.remotes().ok()?;

    if let Some(remote_name) = get_current_branch_upstream_remote(repo) {
        if let Ok(remote) = repo.find_remote(&remote_name) {
            if let Some(result) = f(remote) {
                return Some(result);
            }
        }
    }

    remotes.iter().find_map(|remote_name| {
        let remote = repo.find_remote(remote_name?).ok()?;
        f(remote)
    })
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

pub fn extract_ticket<'b>(branch: &'b str) -> Option<&'b str> {
    let re = Regex::new(r"\w{2,}-\d+").unwrap();
    re.captures(&branch)
        .map(|caps| caps.get(0).unwrap().as_str())
}

pub fn exec_git_cmd<A, I>(args: I, repo: Option<&Repository>) -> Result<(), Error>
where
    A: AsRef<OsStr>,
    I: IntoIterator<Item = A>,
{
    let mut git = Command::new("git");

    if let Some(repo) = repo {
        git.arg(format!("--git-dir={}", repo.path().to_string_lossy()));

        if let Some(workdir) = repo.workdir() {
            git.arg(format!("--work-tree={}", workdir.to_string_lossy()));
        }
    }

    let git = git.args(args);
    // eprintln!(
    //     "\x1b[35mgit {}\x1b[0m",
    //     git.get_args()
    //         .collect::<Vec<_>>()
    //         .join(OsStr::new(" "))
    //         .into_string()
    //         .unwrap_or_default()
    // );

    let output = git
        .spawn()
        .expect("failed to execute process")
        .wait()
        .map_err(Error::FailedToExecuteGit)?;
    if !output.success() {
        exit(output.code().unwrap_or(-1));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempRepo {
        path: std::path::PathBuf,
        repo: Repository,
    }

    impl TempRepo {
        fn new() -> Self {
            let id = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!("gitext-test-{id}"));
            fs::create_dir(&path).unwrap();

            let repo = Repository::init(&path).unwrap();
            repo.set_head("refs/heads/work").unwrap();
            let tree_id = repo.index().unwrap().write_tree().unwrap();
            let tree = repo.find_tree(tree_id).unwrap();
            let signature = git2::Signature::now("gitext", "gitext@example.com").unwrap();
            repo.commit(Some("HEAD"), &signature, &signature, "initial", &tree, &[])
                .unwrap();
            drop(tree);

            Self { path, repo }
        }

        fn set_upstream_remote(&self, remote: &str) {
            let mut config = self.repo.config().unwrap();
            config.set_str("branch.work.remote", remote).unwrap();
            config
                .set_str("branch.work.merge", "refs/heads/work")
                .unwrap();
        }
    }

    impl Drop for TempRepo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn selected_remote_name(repo: &Repository) -> Option<String> {
        find_remote_by_priority(repo, |remote| {
            let url = remote.url()?;

            if url.contains("github.com") {
                remote.name().map(String::from)
            } else {
                None
            }
        })
    }

    #[test]
    fn prefers_current_branch_upstream_remote() {
        let temp = TempRepo::new();
        temp.repo
            .remote("fork", "git@github.com:chipp/gitext-fork.git")
            .unwrap();
        temp.repo
            .remote("origin", "git@github.com:chipp/gitext.git")
            .unwrap();
        temp.set_upstream_remote("origin");

        assert_eq!(selected_remote_name(&temp.repo), Some("origin".to_string()));
    }

    #[test]
    fn can_prefer_non_origin_upstream_remote() {
        let temp = TempRepo::new();
        temp.repo
            .remote("origin", "git@github.com:chipp/gitext.git")
            .unwrap();
        temp.repo
            .remote("fork", "git@github.com:chipp/gitext-fork.git")
            .unwrap();
        temp.set_upstream_remote("fork");

        assert_eq!(selected_remote_name(&temp.repo), Some("fork".to_string()));
    }

    #[test]
    fn falls_back_to_first_matching_remote_without_upstream() {
        let temp = TempRepo::new();
        temp.repo
            .remote("fork", "git@github.com:chipp/gitext-fork.git")
            .unwrap();
        temp.repo
            .remote("origin", "git@github.com:chipp/gitext.git")
            .unwrap();

        assert_eq!(selected_remote_name(&temp.repo), Some("fork".to_string()));
    }

    #[test]
    fn falls_back_when_upstream_remote_does_not_match() {
        let temp = TempRepo::new();
        temp.repo
            .remote("internal", "git@gitlab.company.com:project/gitext.git")
            .unwrap();
        temp.repo
            .remote("origin", "git@github.com:chipp/gitext.git")
            .unwrap();
        temp.repo
            .remote("fork", "git@github.com:chipp/gitext-fork.git")
            .unwrap();
        temp.set_upstream_remote("internal");

        assert_eq!(selected_remote_name(&temp.repo), Some("fork".to_string()));
    }
}
