use super::user::User;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub number: u16,
    pub title: String,

    #[serde(rename = "html_url")]
    pub url: Url,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub merged_at: Option<DateTime<Utc>>,

    pub user: User,
    pub state: State,

    pub head: Ref,
    pub base: Ref,
}

#[derive(Debug, Deserialize)]
pub struct Ref {
    pub label: String,
    pub sha: String,

    #[serde(rename = "ref")]
    pub reference: String,
}

#[derive(Debug, Deserialize, PartialEq, PartialOrd, Eq, Ord)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Open,
    Closed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn json_parsing() {
        let json = serde_json::json!({
            "html_url": "https://github.com/chipp/lisa/pull/18",
            "id": "733000416u64",
            "number": 18u32,
            "state": "closed",
            "title": "Add staging",
            "user": {
                "login": "chipp",
                "id": 123u8
            },
            "created_at": "2021-09-13T18:34:50Z",
            "updated_at": "2022-01-15T21:26:41Z",
            "head": {
                "label": "chipp:add-staging",
                "ref": "add-staging",
                "sha": "5b69861aec37ceb223a563ea85533a988f13fec6",
                "repo": {
                    "id": 262143048u64,
                    "name": "lisa",
                    "full_name": "chipp/lisa",
                    "private": false
                },
                "user": {
                    "login": "chipp",
                    "id": 123u8
                }
            },
            "base": {
                "label": "chipp:main",
                "ref": "main",
                "sha": "25cf604efff9a16fc6db4553cd5075a23bda9a1a",
                "repo": {
                    "id": 262143048u64,
                    "name": "lisa",
                    "full_name": "chipp/lisa",
                    "private": false
                },
                "user": {
                    "login": "chipp",
                    "id": 123u8
                }
            }
        });

        let pr: PullRequest = serde_json::from_value(json).unwrap();

        assert_eq!(pr.number, 18);
        assert_eq!(pr.title, "Add staging");
        assert_eq!(
            pr.created_at,
            Utc.with_ymd_and_hms(2021, 9, 13, 18, 34, 50).unwrap()
        );
        assert_eq!(
            pr.updated_at,
            Utc.with_ymd_and_hms(2022, 1, 15, 21, 26, 41).unwrap()
        );
        assert_eq!(pr.merged_at, None);

        assert_eq!(pr.user.id, 123);
        assert_eq!(pr.user.login, "chipp");
        assert_eq!(pr.user.name, None);

        assert_eq!(pr.state, State::Closed);

        assert_eq!(pr.head.label, "chipp:add-staging");
        assert_eq!(pr.head.sha, "5b69861aec37ceb223a563ea85533a988f13fec6");
        assert_eq!(pr.head.reference, "add-staging");

        assert_eq!(pr.base.label, "chipp:main");
        assert_eq!(pr.base.sha, "25cf604efff9a16fc6db4553cd5075a23bda9a1a");
        assert_eq!(pr.base.reference, "main");

        assert_eq!(
            pr.url,
            Url::parse("https://github.com/chipp/lisa/pull/18").unwrap()
        );
    }
}
