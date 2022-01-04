use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: u16,
    pub name: String,
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
pub struct Actor {
    pub user: User,
    pub approved: bool,
}
