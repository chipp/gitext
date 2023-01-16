use super::user::User;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use url::Url;

#[derive(Deserialize)]
pub struct PullRequest {
    #[serde(rename = "iid")]
    pub id: u16,
    pub title: String,

    #[serde(rename = "web_url")]
    pub url: Url,

    #[serde(rename = "created_at"/*, with = "ts_milliseconds"*/)]
    pub created: DateTime<Utc>,
    #[serde(rename = "updated_at"/*, with = "ts_milliseconds"*/)]
    pub updated: DateTime<Utc>,

    pub author: User,
    pub state: PullRequestState,

    pub sha: String,
    pub source_branch: String,
    pub target_branch: String,

    pub upvotes: u8,
    pub downvotes: u8,
    pub labels: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd, Eq, Ord)]
#[serde(rename_all = "lowercase")]
pub enum PullRequestState {
    Opened,
    Merged,
    Closed,
    Locked,
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::user::User;
    use chrono::{Duration, TimeZone};

    #[test]
    fn parse() {
        let json = serde_json::json!(
          {
            "merge_status": "can_be_merged",
            "author": {
              "username": "vpupkin",
              "id": 10,
              "name": "Vasili Pupkin"
            },
            "created_at": "2021-04-27T16:16:38.490Z",
            "id": 602,
            "state": "opened",
            "has_conflicts": false,
            "description": "- magic description\n- wow",
            "sha": "8b91b4565bc639a1891bd4bbba54442b6647dd23",
            "labels": [
              "2.21",
              "CI OK"
            ],
            "blocking_discussions_resolved": true,
            "source_branch": "some_feature",
            "upvotes": 3,
            "updated_at": "2021-04-27T16:41:32.410Z",
            "target_branch": "develop",
            "downvotes": 1,
            "title": "IOS-1212: Test title",
            "iid": 340,
            "web_url": "https://git.example.com/mr/340"
          }
        );

        let pr: PullRequest = serde_json::from_value(json).unwrap();

        assert_eq!(pr.id, 340);
        assert_eq!(pr.title, "IOS-1212: Test title");
        assert_eq!(
            pr.created,
            Utc.with_ymd_and_hms(2021, 4, 27, 16, 16, 38).unwrap() + Duration::milliseconds(490)
        );
        assert_eq!(
            pr.updated,
            Utc.with_ymd_and_hms(2021, 4, 27, 16, 41, 32).unwrap() + Duration::milliseconds(410)
        );

        assert_eq!(pr.author.id, 10);
        assert_eq!(pr.author.name, "vpupkin");
        assert_eq!(pr.author.display_name, "Vasili Pupkin");
        assert_eq!(pr.state, PullRequestState::Opened);

        assert_eq!(pr.sha, "8b91b4565bc639a1891bd4bbba54442b6647dd23");
        assert_eq!(pr.source_branch, "some_feature");
        assert_eq!(pr.target_branch, "develop");

        assert_eq!(
            pr.url,
            Url::parse("https://git.example.com/mr/340").unwrap()
        );

        assert_eq!(pr.upvotes, 3);
        assert_eq!(pr.downvotes, 1);
        assert_eq!(pr.labels, vec!["2.21", "CI OK"]);
    }
}
