use url::Url;

#[derive(Debug, PartialEq)]
pub struct RepoId {
    pub project: String,
    pub name: String,
}

#[derive(Debug)]
pub struct InvalidRepoId;

impl RepoId {
    pub fn from_str_with_host(remote_url: &str, base_url: &Url) -> Result<RepoId, InvalidRepoId> {
        Self::from_url(remote_url, base_url)
            .or_else(|| Self::from_scp(remote_url, base_url))
            .ok_or(InvalidRepoId)
    }

    pub fn url(&self, base_url: &Url) -> Url {
        let mut url = base_url.clone();

        {
            let mut segments = url.path_segments_mut().unwrap();
            // TODO: handle subgroups
            segments.push(&self.project);
            segments.push(&self.name);
        }

        url
    }

    pub fn id(&self) -> String {
        format!("{}/{}", self.project, self.name)
    }

    fn from_url(url: &str, base_url: &Url) -> Option<RepoId> {
        let url = Url::parse(url).ok()?;

        if let Some(host) = url.host_str() {
            if host != base_url.host_str().unwrap() {
                return None;
            }
        }

        let project;
        let name;

        {
            let mut components = url.path_segments()?;
            project = components.next()?;
            name = components.next()?;
        }

        Some(RepoId {
            project: String::from(project),
            name: String::from(name.trim_end_matches(".git")),
        })
    }

    fn from_scp(url: &str, base_url: &Url) -> Option<RepoId> {
        let (server, path) = crate::split_once!(url, ":")?;

        let host = match crate::split_once!(server, "@") {
            Some((_, host)) => host,
            None => url,
        };

        if host != base_url.host_str().unwrap() {
            return None;
        }

        let project;
        let name;

        {
            let mut path_segments = path.split("/");
            project = path_segments.next()?;
            name = path_segments.next()?;
        }

        Some(RepoId {
            project: String::from(project),
            name: String::from(name.trim_end_matches(".git")),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_url() -> Url {
        Url::parse("https://gitlab.company.com").unwrap()
    }

    #[test]
    fn parse_from_url() {
        assert_eq!(
            RepoId::from_url("ssh://git@gitlab.company.com/project/ios.git", &base_url()),
            Some(RepoId {
                project: "project".to_string(),
                name: "ios".to_string()
            })
        );

        assert_eq!(
            RepoId::from_url("https://gitlab.company.com/project/ios.git", &base_url()),
            Some(RepoId {
                project: "project".to_string(),
                name: "ios".to_string()
            })
        );

        assert_eq!(
            RepoId::from_url("https://gitlab.company.com/ios.git", &base_url()),
            None
        );

        assert_eq!(
            RepoId::from_url("https://invalid.com/project/ios.git", &base_url()),
            None
        );

        assert_eq!(RepoId::from_url("not an url", &base_url()), None);
    }

    #[test]
    fn parse_from_scp_like_url() {
        assert_eq!(
            RepoId::from_scp("git@gitlab.company.com:project/ios.git", &base_url()),
            Some(RepoId {
                project: "project".to_string(),
                name: "ios".to_string()
            })
        );
    }

    #[test]
    fn url() {
        let repo_id = RepoId {
            project: "project".to_string(),
            name: "ios".to_string(),
        };

        assert_eq!(
            repo_id.url(&base_url()).as_str(),
            "https://gitlab.company.com/project/ios"
        )
    }
}
