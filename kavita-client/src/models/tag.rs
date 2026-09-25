use serde::Deserialize;

#[derive(Deserialize, Debug, Default)]
pub struct Tag {
    id: u16,
    title: Option<String>,
}
