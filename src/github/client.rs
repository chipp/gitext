use chipp_http::curl::easy::{self, Auth};
use chipp_http::{Error, HttpClient, Interceptor, Request};
use serde::Serialize;

use crate::git::{AuthDomainConfig, BaseUrlConfig};
use crate::Authenticator;

use super::repo::Repo;
use super::{CheckSuites, PullRequest, RepoId};

pub struct Client<'a> {
    inner: HttpClient<Authenticator<'a>>,
}

impl Client<'_> {
    pub fn new<'a, Conf>(config: &'a Conf) -> Client<'a>
    where
        Conf: BaseUrlConfig,
        Conf: AuthDomainConfig + Send + Sync,
    {
        let mut base_url = config.base_url().clone();
        base_url.set_host(Some("api.github.com")).unwrap();

        let mut inner = HttpClient::new(base_url)
            .unwrap()
            .with_interceptor(Authenticator::basic_auth(config.auth_domain()));

        inner.set_default_headers(&[("User-Agent", "gitext")]);

        Client { inner }
    }
}

impl Client<'_> {
    pub async fn whoami(&self) -> Result<super::user::User, Error> {
        self.inner.get(vec!["user"]).await
    }

    pub async fn find_open_prs(&self, repo_id: &RepoId) -> Result<Vec<PullRequest>, Error> {
        self.inner
            .get_with_params(
                &["repos", &repo_id.owner, &repo_id.repo, "pulls"],
                &[
                    ("state", "open"),
                    ("per_page", "100"),
                    ("sort", "updated"),
                    ("direction", "desc"),
                ],
            )
            .await
    }

    pub async fn get_commit_check_suites(
        &self,
        repo_id: &RepoId,
        commit: &str,
    ) -> Result<CheckSuites, Error> {
        self.inner
            .get_with_params(
                &[
                    "repos",
                    &repo_id.owner,
                    &repo_id.repo,
                    "commits",
                    commit,
                    "check-suites",
                ],
                &[("per_page", "100")],
            )
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
                &["repos", &repo_id.owner, &repo_id.repo, "pulls"],
                &[
                    ("state", state),
                    ("head", &format!("{}:{branch}", repo_id.owner)),
                    ("per_page", "100"),
                    ("sort", "updated"),
                    ("direction", "desc"),
                ],
            )
            .await
    }

    pub async fn get_pr_by_id(&self, pr_id: u16, repo_id: &RepoId) -> Result<PullRequest, Error> {
        self.inner
            .get_with_params(
                &[
                    "repos",
                    &repo_id.owner,
                    &repo_id.repo,
                    "pulls",
                    &format!("{pr_id}"),
                ],
                &[
                    ("state", "open"),
                    ("per_page", "100"),
                    ("sort", "updated"),
                    ("direction", "desc"),
                ],
            )
            .await
    }

    pub async fn get_repo(&self, repo_id: &RepoId) -> Result<Repo, Error> {
        self.inner
            .get(&["repos", &repo_id.owner, &repo_id.repo])
            .await
    }

    pub async fn create_org_repo(
        &self,
        org: &str,
        name: &str,
        private: bool,
    ) -> Result<Repo, Error> {
        #[derive(Serialize)]
        struct CreateBody<'a> {
            name: &'a str,
            private: bool,
        }

        let mut request = self.inner.new_request(&["orgs", org, "repos"]);
        request.set_json_body(&CreateBody { name, private });

        self.inner
            .perform_request(request, chipp_http::json::parse_json)
            .await
    }

    pub async fn create_user_repo(&self, name: &str, private: bool) -> Result<Repo, Error> {
        #[derive(Serialize)]
        struct CreateBody<'a> {
            name: &'a str,
            private: bool,
        }

        let mut request = self.inner.new_request(&["user", "repos"]);
        request.set_json_body(&CreateBody { name, private });

        self.inner
            .perform_request(request, chipp_http::json::parse_json)
            .await
    }
}
