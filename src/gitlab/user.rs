use serde::Deserialize;

#[derive(Deserialize)]
pub struct User {
    pub id: u32,

    #[serde(rename = "username")]
    pub name: String,

    #[serde(rename = "name")]
    pub display_name: String,
}

#[derive(Deserialize)]
pub struct Actor {
    pub user: User,
    pub approved: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parsing() {
        let json = json!({
            "id": 61,
            "name": "Vladimir Burdukov",
            "username": "vladimir_burdukov",
        });

        let user: User = serde_json::from_value(json).unwrap();

        assert_eq!(user.id, 61);
        assert_eq!(user.name, "vladimir_burdukov");
        assert_eq!(user.display_name, "Vladimir Burdukov");
    }
}
