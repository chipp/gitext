use http_client::curl::easy::Auth;
use http_client::{Error, HttpClient};

use crate::common_git::{AuthDomainConfig, BaseUrlConfig};

use super::{CheckSuites, PullRequest, RepoId};

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
        base_url.set_host(Some("api.github.com")).unwrap();

        let mut inner = HttpClient::new(base_url).unwrap();
        inner.set_interceptor(move |easy| {
            easy.ssl_verify_peer(false).unwrap();

            let mut auth = Auth::new();
            auth.basic(true);
            easy.http_auth(&auth).unwrap();

            let (username, password) = auth::user_and_password(config.auth_domain());

            easy.username(username.as_ref()).unwrap();
            easy.password(password.as_ref()).unwrap();
        });

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
                &["repos", &repo_id.user, &repo_id.repo, "pulls"],
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
                    &repo_id.user,
                    &repo_id.repo,
                    "commits",
                    commit,
                    "check-suites",
                ],
                &[("per_page", "100")],
            )
            .await
    }
}
