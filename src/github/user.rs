use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct User {
    pub id: u32,
    pub login: String,
    pub name: Option<String>,
}
