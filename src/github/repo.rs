use serde::Deserialize;

use super::user::User;

#[derive(Debug, Deserialize)]
pub struct Repo {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub owner: User,
    pub html_url: String,
    pub ssh_url: String,
    pub private: bool,
}
