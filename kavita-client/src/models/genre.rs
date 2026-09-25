use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
pub struct GenreTag {
    pub id: u16,
    pub name: Option<String>,
}
