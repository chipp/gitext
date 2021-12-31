use crate::common_git::{AuthDomainConfig, JiraUrlConfig};
use crate::error::Error;
use http_client::{curl::easy::Auth, Error as HttpError, HttpClient};
use serde::Deserialize;

pub struct JiraClient<'a> {
    inner: HttpClient<'a>,
}

impl JiraClient<'_> {
    pub fn new<'a, Conf>(config: &'a Conf) -> Result<JiraClient<'a>, Error>
    where
        Conf: AuthDomainConfig + Send + Sync,
        Conf: JiraUrlConfig,
    {
        let jira_url = config.jira_url().ok_or(Error::JiraUrlNotConfigured)?;

        let mut inner = HttpClient::new(jira_url).unwrap();
        inner.set_interceptor(move |easy| {
            let mut auth = Auth::new();
            auth.basic(true);
            easy.http_auth(&auth).unwrap();

            let (username, password) = auth::user_and_password(config.auth_domain());

            easy.username(username.as_ref()).unwrap();
            easy.password(password.as_ref()).unwrap();
        });

        Ok(JiraClient { inner })
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
    ) -> Result<IssuesPageResponse, HttpError> {
        let mut params = vec![];

        params.push(("startAt", format!("{}", start_at)));
        params.push(("maxResults", format!("{}", max_results)));
        params.push(("jql", jql));

        if let Some(fields) = fields {
            params.push(("fields", fields.join(",")));
        }

        let mut request = self
            .inner
            .new_request_with_params(vec!["rest", "api", "2", "search"], &params);
        request.set_retry_count(3);

        self.inner
            .perform_request(request, http_client::json::parse_json)
            .await
    }
}
