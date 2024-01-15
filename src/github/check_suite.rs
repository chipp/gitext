use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CheckSuites {
    pub total_count: u8,
    pub check_suites: Vec<CheckSuite>,
}

#[derive(Deserialize)]
pub struct CheckSuite {
    pub id: u64,
    pub status: Status,
    pub conclusion: Option<Conclusion>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Queued,
    InProgress,
    Completed,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Conclusion {
    Success,
    Failure,
    Neutral,
    Cancelled,
    Skipped,
    TimedOut,
    ActionRequired,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn json_parsing() {
        let json = serde_json::json!({
            "total_count": 3,
            "check_suites": [
                {
                    "id": 1u8,
                    "status": "completed",
                    "conclusion": "success",
                    "created_at": "2021-09-14T11:45:36Z",
                    "updated_at": "2021-09-14T11:50:40Z",
                },
                {
                    "id": 2u8,
                    "status": "completed",
                    "conclusion": "failure",
                    "created_at": "2021-10-15T9:10:21Z",
                    "updated_at": "2021-10-15T9:13:31Z",
                },
                {
                    "id": 3u8,
                    "status": "in_progress",
                    "conclusion": null,
                    "created_at": "2022-01-16T10:17:24Z",
                    "updated_at": "2022-01-16T10:17:33Z",
                }
            ]
        });

        let check_suites: CheckSuites = serde_json::from_value(json).unwrap();

        assert_eq!(check_suites.total_count, 3);

        {
            let check_suite = check_suites.check_suites.get(0).unwrap();
            assert_eq!(check_suite.id, 1);
            assert_eq!(check_suite.status, Status::Completed);
            assert_eq!(check_suite.conclusion, Some(Conclusion::Success));
            assert_eq!(
                check_suite.created_at,
                Utc.with_ymd_and_hms(2021, 9, 14, 11, 45, 36).unwrap()
            );
            assert_eq!(
                check_suite.updated_at,
                Some(Utc.with_ymd_and_hms(2021, 9, 14, 11, 50, 40).unwrap())
            );
        }

        {
            let check_suite = check_suites.check_suites.get(1).unwrap();
            assert_eq!(check_suite.id, 2);
            assert_eq!(check_suite.status, Status::Completed);
            assert_eq!(check_suite.conclusion, Some(Conclusion::Failure));
            assert_eq!(
                check_suite.created_at,
                Utc.with_ymd_and_hms(2021, 10, 15, 9, 10, 21).unwrap()
            );
            assert_eq!(
                check_suite.updated_at,
                Some(Utc.with_ymd_and_hms(2021, 10, 15, 9, 13, 31).unwrap())
            );
        }

        {
            let check_suite = check_suites.check_suites.get(2).unwrap();
            assert_eq!(check_suite.id, 3);
            assert_eq!(check_suite.status, Status::InProgress);
            assert_eq!(check_suite.conclusion, None);
            assert_eq!(
                check_suite.created_at,
                Utc.with_ymd_and_hms(2022, 1, 16, 10, 17, 24).unwrap()
            );
            assert_eq!(
                check_suite.updated_at,
                Some(Utc.with_ymd_and_hms(2022, 1, 16, 10, 17, 33).unwrap())
            );
        }
    }
}
