use url::Url;

pub const SERVER_URL: &str = "https://bitbucket.company.com";
const SERVER_HOST: &str = "bitbucket.company.com";

#[derive(Debug, PartialEq)]
pub struct RepoId {
    pub project: String,
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct InvalidRepoId;

impl std::str::FromStr for RepoId {
    type Err = InvalidRepoId;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_url(s)
            .or_else(|| Self::from_scp(s))
            .ok_or(InvalidRepoId)
    }
}

impl RepoId {
    fn from_url(url: &str) -> Option<RepoId> {
        let url = Url::parse(url).ok()?;

        if let Some(host) = url.host_str() {
            if host != SERVER_HOST {
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

    fn from_scp(url: &str) -> Option<RepoId> {
        let (server, path) = crate::split_once!(url, ":")?;

        let host = match crate::split_once!(server, "@") {
            Some((_, host)) => host,
            None => url,
        };

        if host != SERVER_HOST {
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

    pub fn url(&self) -> Url {
        let mut url = Url::parse(SERVER_URL).unwrap();

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
    use std::str::FromStr;

    use super::*;

    #[test]
    fn parse_from_url() {
        assert_eq!(
            RepoId::from_url("ssh://git@bitbucket.company.com:7999/ap/mobile_ios.git"),
            Some(RepoId {
                project: "ap".to_string(),
                name: "mobile_ios".to_string()
            })
        );

        assert_eq!(
            RepoId::from_url("https://bitbucket.company.com/scm/ap/mobile_ios.git"),
            Some(RepoId {
                project: "ap".to_string(),
                name: "mobile_ios".to_string()
            })
        );

        assert_eq!(
            RepoId::from_url("https://bitbucket.company.com/mobile_ios.git"),
            None
        );

        assert_eq!(
            RepoId::from_url("https://invalid.com/scm/ap/mobile_ios.git"),
            None
        );

        assert_eq!(RepoId::from_url("not an url"), None);
    }

    #[test]
    fn parse_from_scp_like_url() {
        assert_eq!(
            RepoId::from_scp("git@bitbucket.company.com:ap/mobile_ios.git"),
            Some(RepoId {
                project: "ap".to_string(),
                name: "mobile_ios".to_string()
            })
        );
    }

    #[test]
    fn parse_from_str() {
        assert_eq!(
            RepoId::from_str("ssh://git@bitbucket.company.com:7999/ap/mobile_ios.git"),
            Ok(RepoId {
                project: "ap".to_string(),
                name: "mobile_ios".to_string()
            })
        );

        assert_eq!(
            RepoId::from_str("https://bitbucket.company.com/scm/ap/mobile_ios.git"),
            Ok(RepoId {
                project: "ap".to_string(),
                name: "mobile_ios".to_string()
            })
        );

        assert_eq!(
            RepoId::from_str("git@bitbucket.company.com:ap/mobile_ios.git"),
            Ok(RepoId {
                project: "ap".to_string(),
                name: "mobile_ios".to_string()
            })
        );

        assert_eq!(
            RepoId::from_str("https://invalid.com/scm/ap/mobile_ios.git"),
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
            repo_id.url().as_str(),
            "https://bitbucket.company.com/projects/AP/repos/mobile_ios"
        )
    }
}
