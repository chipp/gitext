use crate::git::{AuthDomainConfig, BaseUrlConfig};

use super::{user::User, Pipeline, PullRequest, RepoId};

use chipp_http::{Error, HttpClient};

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
                chipp_auth::token(config.auth_domain(), "access_token")
            ),
        )]);

        Client { inner }
    }
}

impl Client<'_> {
    pub async fn whoami(&self) -> Result<User, Error> {
        self.inner.get(vec!["user"]).await
    }

    pub async fn get_user_by_name<N: AsRef<str>>(&self, name: N) -> Result<Vec<User>, Error> {
        self.inner
            .get_with_params(vec!["users"], [("username", name)])
            .await
    }

    pub async fn find_open_prs(
        &self,
        repo_id: &RepoId,
        author: Option<u32>,
        assignee: Option<u32>,
        page: u8,
    ) -> Result<Vec<PullRequest>, Error> {
        let page = format!("{}", page);
        let mut params = vec![("state", "opened"), ("page", &page)];

        let author = author.map(|a| format!("{}", a));
        let assignee = assignee.map(|a| format!("{}", a));

        if let Some(author) = author.as_ref() {
            params.push(("author_id", author));
        }

        if let Some(assignee) = assignee.as_ref() {
            params.push(("assignee_id", assignee));
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
