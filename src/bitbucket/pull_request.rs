use super::user::Actor;
use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequest {
    pub id: u16,
    pub title: String,

    #[serde(rename = "createdDate", with = "ts_milliseconds")]
    pub created: DateTime<Utc>,
    #[serde(rename = "updatedDate", with = "ts_milliseconds")]
    pub updated: DateTime<Utc>,

    pub author: Actor,
    pub reviewers: Vec<Actor>,

    pub from_ref: Ref,
    pub to_ref: Ref,
    pub state: PullRequestState,
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd, Eq, Ord)]
#[serde(rename_all = "UPPERCASE")]
pub enum PullRequestState {
    Open,
    Merged,
    Declined,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ref {
    pub display_id: String,
    pub id: String,
    pub latest_commit: String,
    pub repository: Repository,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    pub slug: String,
    pub id: u16,
    pub project: Project,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub key: String,
    pub id: u16,
}

impl PullRequest {
    pub fn url(&self, base_url: &Url) -> Url {
        let mut url = base_url.clone();

        {
            let mut segments = url.path_segments_mut().unwrap();
            segments.push("projects");
            segments.push(&self.to_ref.repository.project.key.to_uppercase());
            segments.push("repos");
            segments.push(&self.to_ref.repository.slug);
            segments.push("pull-requests");
            segments.push(&self.id.to_string());
        }

        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitbucket::user::User;

    #[test]
    fn url() {
        let pr = PullRequest {
            id: 42,
            from_ref: Ref {
                display_id: "_".to_string(),
                id: "_".to_string(),
                latest_commit: "1".to_string(),
                repository: Repository {
                    slug: "gitbucket".to_string(),
                    id: 42,
                    project: Project {
                        key: "~vburduko".to_string(),
                        id: 1,
                    },
                },
            },
            to_ref: Ref {
                display_id: "_".to_string(),
                id: "_".to_string(),
                latest_commit: "1".to_string(),
                repository: Repository {
                    slug: "gitbucket".to_string(),
                    id: 42,
                    project: Project {
                        key: "VB".to_string(),
                        id: 42,
                    },
                },
            },
            state: PullRequestState::Open,
            title: String::default(),
            author: Actor {
                user: User {
                    id: 1,
                    name: String::default(),
                    display_name: String::default(),
                },
                approved: false,
            },
            reviewers: vec![],
            created: Utc::now(),
            updated: Utc::now(),
        };

        assert_eq!(
            pr.url(&Url::parse("https://bitbucket.company.com").unwrap())
                .as_str(),
            "https://bitbucket.company.com/projects/VB/repos/gitbucket/pull-requests/42"
        )
    }
}
