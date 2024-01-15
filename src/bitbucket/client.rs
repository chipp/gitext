use super::repo::Repo;
use super::{PullRequest, RepoId};
use crate::common_git::{AuthDomainConfig, BaseUrlConfig};

use http_client::curl::easy::Auth;
use http_client::json::parse_json;
use http_client::{Error, HttpClient};
use serde::{Deserialize, Serialize};

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
        base_url.set_path("/rest/api/1.0/");

        let mut inner = HttpClient::new(base_url).unwrap();
        inner.set_interceptor(move |easy| {
            let mut auth = Auth::new();
            auth.basic(true);
            easy.http_auth(&auth).unwrap();

            let (username, password) = auth::user_and_password(config.auth_domain());

            easy.username(username.as_ref()).unwrap();
            easy.password(password.as_ref()).unwrap();
        });

        Client { inner }
    }
}

impl Client<'_> {
    pub async fn whoami(&self, username: &str) -> Result<super::user::User, Error> {
        self.inner.get(vec!["users", username]).await
    }

    pub async fn create_repo(&self, repo_id: RepoId) -> Result<Repo, Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct CreateBody {
            name: String,
            scm_id: &'static str,
        }

        let mut request = self
            .inner
            .new_request(&["projects", &repo_id.project, "repos"]);

        let body = CreateBody {
            name: repo_id.name,
            scm_id: "git",
        };

        request.set_json_body(&body);

        if let Some(ref body) = request.body {
            println!("body {}", String::from_utf8_lossy(&body));
        }

        self.inner.perform_request(request, parse_json).await
    }

    pub async fn find_open_prs(
        &self,
        repo_id: &RepoId,
        author: Option<String>,
    ) -> Result<PageResponse<PullRequest>, Error> {
        let params: Vec<(&str, &str)>;
        if let Some(author) = author.as_ref() {
            params = vec![("username.1", author), ("role.1", "AUTHOR")];
        } else {
            params = vec![];
        }

        self.inner
            .get_with_params(
                vec![
                    "projects",
                    &repo_id.project,
                    "repos",
                    &repo_id.name,
                    "pull-requests",
                ],
                &params,
            )
            .await
    }

    pub async fn find_prs_for_branch(
        &self,
        branch: &str,
        repo_id: &RepoId,
        state: &str,
    ) -> Result<Vec<PullRequest>, Error> {
        // TODO: find another way to get full branch identifier
        let branch = format!("refs/heads/{}", branch);

        let response = self
            .inner
            .get_with_params(
                vec![
                    "projects",
                    &repo_id.project,
                    "repos",
                    &repo_id.name,
                    "pull-requests",
                ],
                &[
                    ("at", branch.as_str()),
                    ("direction", "OUTGOING"),
                    ("state", state),
                ],
            )
            .await;

        response.map(|r: PageResponse<PullRequest>| r.values)
    }

    pub async fn get_pr_by_id(&self, id: u16, repo_id: &RepoId) -> Result<PullRequest, Error> {
        self.inner
            .get(vec![
                "projects",
                &repo_id.project,
                "repos",
                &repo_id.name,
                "pull-requests",
                &id.to_string(),
            ])
            .await
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageResponse<V> {
    pub values: Vec<V>,
    pub is_last_page: bool,
    pub size: u16,
    pub start: u16,
    pub limit: u16,
}
