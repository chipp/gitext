use url::Url;

#[derive(Clone, Debug, PartialEq)]
pub struct RepoId {
    pub owner: String,
    pub repo: String,
}

#[derive(Debug, PartialEq)]
pub struct InvalidRepoId;

impl RepoId {
    pub fn from_str_with_host(remote_url: &str, base_url: &Url) -> Result<RepoId, InvalidRepoId> {
        Self::from_url(remote_url, base_url)
            .or_else(|| Self::from_scp(remote_url, base_url))
            .ok_or(InvalidRepoId)
    }

    fn from_url(url: &str, base_url: &Url) -> Option<RepoId> {
        let url = Url::parse(url).ok()?;

        if let Some(host) = url.host_str() {
            if host != base_url.host_str().unwrap() {
                return None;
            }
        }

        let repo;
        let owner;

        {
            let mut components = url.path_segments()?.rev().take(2);
            repo = components.next()?;
            owner = components.next()?;
        }

        Some(RepoId {
            owner: String::from(owner),
            repo: String::from(repo.trim_end_matches(".git")),
        })
    }

    fn from_scp(url: &str, base_url: &Url) -> Option<RepoId> {
        let (server, path) = url.split_once(":")?;

        let host = match server.split_once("@") {
            Some((_, host)) => host,
            None => url,
        };

        if host != base_url.host_str().unwrap() {
            return None;
        }

        let owner;
        let repo;

        {
            let mut path_segments = path.split("/");
            owner = path_segments.next()?;
            repo = path_segments.next()?;
        }

        Some(RepoId {
            owner: String::from(owner),
            repo: String::from(repo.trim_end_matches(".git")),
        })
    }

    pub fn url(&self, base_url: &Url) -> Url {
        let mut url = base_url.clone();

        {
            let mut segments = url.path_segments_mut().unwrap();
            segments.push(&self.owner);
            segments.push(&self.repo);
        }

        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_url() -> Url {
        Url::parse("https://github.com").unwrap()
    }

    #[test]
    fn parse_from_url() {
        assert_eq!(
            RepoId::from_url("https://github.com/chipp/gitext.git", &base_url()),
            Some(RepoId {
                owner: "chipp".to_string(),
                repo: "gitext".to_string()
            })
        );

        assert_eq!(
            RepoId::from_url("https://github.com/gitext.git", &base_url()),
            None
        );

        assert_eq!(
            RepoId::from_url("https://invalid.com/chipp/gitext.git", &base_url()),
            None
        );

        assert_eq!(RepoId::from_url("not an url", &base_url()), None);
    }

    #[test]
    fn parse_from_scp_like_url() {
        assert_eq!(
            RepoId::from_scp("git@github.com:chipp/gitext.git", &base_url()),
            Some(RepoId {
                owner: "chipp".to_string(),
                repo: "gitext".to_string()
            })
        );
    }

    #[test]
    fn parse_from_str() {
        assert_eq!(
            RepoId::from_str_with_host("https://github.com/chipp/gitext.git", &base_url()),
            Ok(RepoId {
                owner: "chipp".to_string(),
                repo: "gitext".to_string()
            })
        );

        assert_eq!(
            RepoId::from_str_with_host("git@github.com:chipp/gitext.git", &base_url()),
            Ok(RepoId {
                owner: "chipp".to_string(),
                repo: "gitext".to_string()
            })
        );

        assert_eq!(
            RepoId::from_str_with_host("https://invalid.com/chipp/gitext.git", &base_url()),
            Err(InvalidRepoId)
        );
    }

    #[test]
    fn url() {
        let repo_id = RepoId {
            owner: "chipp".to_string(),
            repo: "gitext".to_string(),
        };

        assert_eq!(
            repo_id.url(&base_url()).as_str(),
            "https://github.com/chipp/gitext"
        )
    }
}
