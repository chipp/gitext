use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Repo {
    pub slug: String,
    pub id: u16,
    pub project: Project,
    pub links: RepoLinks,
}

#[derive(Debug, Deserialize)]
pub struct RepoLinks {
    pub clone: Vec<Link>,

    #[serde(rename = "self")]
    pub self_: Vec<Link>,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub key: String,
    pub id: u16,
}

#[derive(Debug, Deserialize)]
pub struct Link {
    pub href: String,
    pub name: Option<String>,
}
