use serde::Deserialize;

use crate::models::chapter::Chapter;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Volume {
    pub id: u16,
    pub name: String,
    pub cover_image: String,
    pub pages: u16,
    pub pages_read: u16,
    pub series_id: u16,
    pub min_hours_to_read: u8,
    pub max_hours_to_read: u8,
    pub avg_hours_to_read: f32,
    pub chapters: Vec<Chapter>,
}
