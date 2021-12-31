use url::Url;

#[derive(Debug, PartialEq)]
pub struct RepoId {
    pub project: String,
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct InvalidRepoId;

impl RepoId {
    pub fn from_str_with_host(
        remote_url: &str,
        config_base_url: &Url,
    ) -> Result<RepoId, InvalidRepoId> {
        Self::from_url(remote_url, config_base_url)
            .or_else(|| Self::from_scp(remote_url, config_base_url))
            .ok_or(InvalidRepoId)
    }

    fn from_url(url: &str, config_base_url: &Url) -> Option<RepoId> {
        let url = Url::parse(url).ok()?;

        if let Some(host) = url.host_str() {
            if host != config_base_url.host_str().unwrap() {
                return None;
            }
        }

        let name;
        let project;

        {
            let mut components = url.path_segments()?.rev().take(2);
            name = components.next()?;
            project = components.next()?;
        }

        Some(RepoId {
            project: String::from(project),
            name: String::from(name.trim_end_matches(".git")),
        })
    }

    fn from_scp(url: &str, config_base_url: &Url) -> Option<RepoId> {
        let (server, path) = crate::split_once!(url, ":")?;

        let host = match crate::split_once!(server, "@") {
            Some((_, host)) => host,
            None => url,
        };

        if host != config_base_url.host_str().unwrap() {
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

    pub fn url(&self, base_url: &Url) -> Url {
        let mut url = base_url.clone();

        {
            let mut segments = url.path_segments_mut().unwrap();
            segments.push("projects");
            segments.push(&self.project.to_uppercase());
            segments.push("repos");
            segments.push(&self.name);
        }

        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_url() -> Url {
        Url::parse("https://bitbucket.company.com").unwrap()
    }

    #[test]
    fn parse_from_url() {
        assert_eq!(
            RepoId::from_url(
                "ssh://git@bitbucket.company.com:7999/ap/mobile_ios.git",
                &base_url()
            ),
            Some(RepoId {
                project: "ap".to_string(),
                name: "mobile_ios".to_string()
            })
        );

        assert_eq!(
            RepoId::from_url(
                "https://bitbucket.company.com/scm/ap/mobile_ios.git",
                &base_url()
            ),
            Some(RepoId {
                project: "ap".to_string(),
                name: "mobile_ios".to_string()
            })
        );

        assert_eq!(
            RepoId::from_url("https://bitbucket.company.com/mobile_ios.git", &base_url()),
            None
        );

        assert_eq!(
            RepoId::from_url("https://invalid.com/scm/ap/mobile_ios.git", &base_url()),
            None
        );

        assert_eq!(RepoId::from_url("not an url", &base_url()), None);
    }

    #[test]
    fn parse_from_scp_like_url() {
        assert_eq!(
            RepoId::from_scp("git@bitbucket.company.com:ap/mobile_ios.git", &base_url()),
            Some(RepoId {
                project: "ap".to_string(),
                name: "mobile_ios".to_string()
            })
        );
    }

    #[test]
    fn parse_from_str() {
        assert_eq!(
            RepoId::from_str_with_host(
                "ssh://git@bitbucket.company.com:7999/ap/mobile_ios.git",
                &base_url()
            ),
            Ok(RepoId {
                project: "ap".to_string(),
                name: "mobile_ios".to_string()
            })
        );

        assert_eq!(
            RepoId::from_str_with_host(
                "https://bitbucket.company.com/scm/ap/mobile_ios.git",
                &base_url()
            ),
            Ok(RepoId {
                project: "ap".to_string(),
                name: "mobile_ios".to_string()
            })
        );

        assert_eq!(
            RepoId::from_str_with_host("git@bitbucket.company.com:ap/mobile_ios.git", &base_url()),
            Ok(RepoId {
                project: "ap".to_string(),
                name: "mobile_ios".to_string()
            })
        );

        assert_eq!(
            RepoId::from_str_with_host("https://invalid.com/scm/ap/mobile_ios.git", &base_url()),
            Err(InvalidRepoId)
        );
    }

    #[test]
    fn url() {
        let repo_id = RepoId {
            project: "ap".to_string(),
            name: "mobile_ios".to_string(),
        };

        assert_eq!(
            repo_id.url(&base_url()).as_str(),
            "https://bitbucket.company.com/projects/AP/repos/mobile_ios"
        )
    }
}
