use http_client::{curl::easy::Auth, Error, HttpClient};
use serde::Deserialize;

pub struct JiraClient<'a> {
    inner: HttpClient<'a>,
}

impl<'a> JiraClient<'a> {
    pub fn new() -> JiraClient<'a> {
        let mut inner = HttpClient::new("https://jira.company.com/rest").unwrap();

        let (username, password) = auth::credentials();

        inner.set_interceptor(move |easy| {
            let mut auth = Auth::new();

            auth.basic(true);

            easy.http_auth(&auth).unwrap();

            easy.username(&username).unwrap();
            easy.password(&password).unwrap();
        });

        JiraClient { inner }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Issue {
    pub id: String,
    pub key: String,
    pub fields: Fields,
}

use std::fmt;
impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.key)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Fields {
    pub status: IssueStatus,
}

#[derive(Clone, Debug, Deserialize)]
pub struct IssueStatus {
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuesPageResponse {
    pub max_results: u16,
    pub total: u16,
    pub issues: Vec<Issue>,
}

impl JiraClient<'_> {
    pub async fn search_issues(
        &self,
        jql: String,
        start_at: u16,
        max_results: u16,
        fields: Option<&[&str]>,
    ) -> Result<IssuesPageResponse, Error> {
        let mut params = vec![];

        params.push(("startAt", format!("{}", start_at)));
        params.push(("maxResults", format!("{}", max_results)));
        params.push(("jql", jql));

        if let Some(fields) = fields {
            params.push(("fields", fields.join(",")));
        }

        let mut request = self
            .inner
            .new_request_with_params(vec!["api", "2", "search"], &params);
        request.set_retry_count(3);

        self.inner
            .perform_request(request, http_client::json::parse_json)
            .await
    }
}
