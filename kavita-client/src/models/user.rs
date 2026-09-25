use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: u16,
    pub username: Option<String>,
    pub email: Option<String>,
    pub roles: Vec<String>,
    pub token: Option<String>,
    pub refresh_token: Option<String>,
    pub api_key: Option<String>,
}
