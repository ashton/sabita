use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
pub struct Tag {
    pub id: u16,
    pub title: Option<String>,
}
