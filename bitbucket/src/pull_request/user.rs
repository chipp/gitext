use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: u16,
    pub name: String,
    pub display_name: String,
}

#[derive(Deserialize)]
pub struct Actor {
    pub user: User,
    pub approved: bool,
}
