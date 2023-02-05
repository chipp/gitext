use url::Url;

#[derive(Debug, PartialEq)]
pub struct RepoId {
    project_path: Vec<String>,
    name: String,
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
            segments.extend(self.project_path.iter());
            segments.push(&self.name);
        }

        url
    }

    pub fn id(&self) -> String {
        let mut components = self.project_path.clone();
        components.push(self.name.clone());

        components.join("/")
    }

    fn from_url(url: &str, base_url: &Url) -> Option<RepoId> {
        let url = Url::parse(url).ok()?;

        if let Some(host) = url.host_str() {
            if host != base_url.host_str().unwrap() {
                return None;
            }
        }

        let mut project_path = url.path_segments()?.collect::<Vec<_>>();
        let name = project_path.pop()?;

        if project_path.is_empty() {
            return None;
        }

        let project_path = project_path.into_iter().map(String::from).collect();

        Some(RepoId {
            project_path,
            name: String::from(name.trim_end_matches(".git")),
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

        let mut project_path = path.split("/").collect::<Vec<_>>();
        let name = project_path.pop()?;

        if project_path.is_empty() {
            return None;
        }

        let project_path = project_path.into_iter().map(String::from).collect();

        Some(RepoId {
            project_path,
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
                project_path: vec!["project".to_string()],
                name: "ios".to_string()
            })
        );

        assert_eq!(
            RepoId::from_url("https://gitlab.company.com/project/ios.git", &base_url()),
            Some(RepoId {
                project_path: vec!["project".to_string()],
                name: "ios".to_string()
            })
        );

        assert_eq!(
            RepoId::from_url(
                "ssh://git@gitlab.company.com/project/subgroup1/subgroup2/ios.git",
                &base_url()
            ),
            Some(RepoId {
                project_path: vec![
                    "project".to_string(),
                    "subgroup1".to_string(),
                    "subgroup2".to_string(),
                ],
                name: "ios".to_string()
            })
        );

        assert_eq!(
            RepoId::from_url(
                "https://gitlab.company.com/project/subgroup1/subgroup2/ios.git",
                &base_url()
            ),
            Some(RepoId {
                project_path: vec![
                    "project".to_string(),
                    "subgroup1".to_string(),
                    "subgroup2".to_string(),
                ],
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
                project_path: vec!["project".to_string()],
                name: "ios".to_string()
            })
        );

        assert_eq!(
            RepoId::from_scp(
                "git@gitlab.company.com:project/subgroup1/subgroup2/ios.git",
                &base_url()
            ),
            Some(RepoId {
                project_path: vec![
                    "project".to_string(),
                    "subgroup1".to_string(),
                    "subgroup2".to_string(),
                ],
                name: "ios".to_string()
            })
        );

        assert_eq!(
            RepoId::from_scp("git@gitlab.company.com:ios.git", &base_url()),
            None
        );
    }

    #[test]
    fn url() {
        let repo_id = RepoId {
            project_path: vec!["project".to_string()],
            name: "ios".to_string(),
        };

        assert_eq!(
            repo_id.url(&base_url()).as_str(),
            "https://gitlab.company.com/project/ios"
        );

        let repo_id = RepoId {
            project_path: vec![
                "project".to_string(),
                "subgroup1".to_string(),
                "subgroup2".to_string(),
            ],
            name: "ios".to_string(),
        };

        assert_eq!(
            repo_id.url(&base_url()).as_str(),
            "https://gitlab.company.com/project/subgroup1/subgroup2/ios"
        );
    }

    #[test]
    fn id() {
        let repo_id = RepoId {
            project_path: vec!["project".to_string()],
            name: "ios".to_string(),
        };

        assert_eq!(repo_id.id().as_str(), "project/ios");

        let repo_id = RepoId {
            project_path: vec![
                "group".to_string(),
                "subgroup1".to_string(),
                "subgroup2".to_string(),
            ],
            name: "ios".to_string(),
        };

        assert_eq!(repo_id.id().as_str(), "group/subgroup1/subgroup2/ios");
    }
}
