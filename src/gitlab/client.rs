use crate::common_git::{AuthDomainConfig, BaseUrlConfig};

use super::{Pipeline, PullRequest, RepoId};

use http_client::{Error, HttpClient};

pub struct Client<'a> {
    inner: HttpClient<'a>,
}

impl Client<'_> {
    pub fn new<'a, Conf>(config: &'a Conf) -> Client<'a>
    where
        Conf: BaseUrlConfig,
        Conf: AuthDomainConfig + Send + Sync,
    {
        let mut base_url = config.base_url().clone();
        base_url.set_path("/api/v4/");

        let mut inner = HttpClient::new(base_url).unwrap();

        inner.set_default_headers(&[(
            "Authorization",
            &format!(
                "Bearer {}",
                auth::token(config.auth_domain(), "access_token")
            ),
        )]);

        Client { inner }
    }
}

impl Client<'_> {
    pub async fn whoami(&self) -> Result<super::user::User, Error> {
        self.inner.get(vec!["user"]).await
    }

    pub async fn find_open_prs<A: AsRef<str>>(
        &self,
        repo_id: &RepoId,
        author: Option<A>,
        page: u8,
    ) -> Result<Vec<PullRequest>, Error> {
        let page = format!("{}", page);
        let mut params = vec![("state", "opened"), ("page", &page)];

        if let Some(author) = author.as_ref() {
            params.push(("author_username", author.as_ref()));
        }

        self.inner
            .get_with_params(vec!["projects", &repo_id.id(), "merge_requests"], &params)
            .await
    }

    pub async fn find_prs_for_branch(
        &self,
        branch: &str,
        repo_id: &RepoId,
        state: &str,
    ) -> Result<Vec<PullRequest>, Error> {
        self.inner
            .get_with_params(
                vec!["projects", &repo_id.id(), "merge_requests"],
                &[("source_branch", branch), ("state", state)],
            )
            .await
    }

    pub async fn get_last_pipeline_for_branch(
        &self,
        branch: &str,
        repo_id: &RepoId,
    ) -> Result<Pipeline, Error> {
        self.inner
            .get_with_params(
                vec!["projects", &repo_id.id(), "pipelines", "latest"],
                &[("ref", branch)],
            )
            .await
    }

    pub async fn get_pr_by_id(&self, id: u16, repo_id: &RepoId) -> Result<PullRequest, Error> {
        self.inner
            .get(vec![
                "projects",
                &repo_id.id(),
                "merge_requests",
                &id.to_string(),
            ])
            .await
    }
}
