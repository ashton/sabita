use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
pub struct GenreTag {
    id: u16,
    name: Option<String>,
}
