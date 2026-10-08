use chrono::NaiveDateTime;
use serde::Deserialize;
use serde_repr::Deserialize_repr;

#[derive(Deserialize_repr, Debug, Default, PartialEq)]
#[repr(u8)]
pub enum SeriesFormat {
    Image = 0,
    #[default]
    Archive = 1,
    Unknown = 2,
    Epub = 3,
    Pdf = 4,
}

#[derive(Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Series {
    pub id: u16,
    pub name: String,
    pub original_name: String,
    pub localized_name: String,
    pub cover_image: String,
    pub sort_name: String,
    pub pages: u16,
    pub user_rating: f32,
    pub has_user_rated: bool,
    pub total_reads: u16,
    pub pages_read: u16,
    pub latest_read_date: NaiveDateTime,
    pub format: SeriesFormat,
    pub library_id: u16,
    pub library_name: String,
    pub min_hours_to_read: u8,
    pub max_hours_to_read: u8,
    pub avg_hours_to_read: f32,
    pub folder_path: String,
    /// Kavita's JSON key is `isBlacklisted` (lowercase "l"), not the
    /// `isBlackListed` that `is_black_listed` would camelCase to — with
    /// `#[serde(default)]` the mismatch wouldn't error, just silently leave
    /// this always `false`.
    #[serde(rename = "isBlacklisted")]
    pub is_black_listed: bool,
}
