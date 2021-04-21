use crate::{PullRequest, RepoId};
use http_client::curl::easy::Auth;
use http_client::{Error, HttpClient};
use serde::Deserialize;

pub struct Client<'a> {
    inner: HttpClient<'a>,
}

const SERVER_URL: &str = "https://bitbucket.company.com/rest/api/1.0/";

impl Client<'_> {
    pub fn new<'a>() -> Client<'a> {
        let (username, password) = auth::credentials();

        let mut inner = HttpClient::new(SERVER_URL).unwrap();
        inner.set_interceptor(move |easy| {
            let mut auth = Auth::new();
            auth.basic(true);
            easy.http_auth(&auth).unwrap();

            easy.username(username.as_ref()).unwrap();
            easy.password(password.as_ref()).unwrap();
        });

        Client { inner }
    }
}

impl Client<'_> {
    pub async fn find_prs_for_branch(
        &self,
        branch: &str,
        repo_id: &RepoId,
    ) -> Result<Vec<PullRequest>, Error> {
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
                &[("at", branch), ("direction", "OUTGOING"), ("state", "ALL")],
            )
            .await;

        response.map(|r: PageResponse<PullRequest>| r.values)
    }
}

#[derive(Deserialize)]
struct PageResponse<V> {
    values: Vec<V>,
}