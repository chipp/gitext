use url::Url;

pub const SERVER_URL: &str = "https://bitbucket.company.com";
const SERVER_HOST: &str = "bitbucket.company.com";

#[derive(Debug, PartialEq)]
pub struct RepoId {
    pub project: String,
    pub name: String,
}

impl RepoId {
    pub fn from_url(url: &str) -> Option<RepoId> {
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
