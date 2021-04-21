use serde::Deserialize;
use url::Url;

#[derive(Deserialize)]
pub struct PullRequest {
    pub id: i16,
}

impl PullRequest {
    pub fn url(&self) -> Url {
        unimplemented!();
    }
}
