use chrono::NaiveTime;
use serde::Deserialize;
use serde_repr::Deserialize_repr;

#[derive(Deserialize_repr, Debug, Default, PartialEq)]
#[repr(u8)]
pub enum KavitaFileType {
    #[default]
    Archive = 1,
    EPub = 2,
    Pdf = 3,
    Images = 4,
}

#[derive(Deserialize, Debug, Default)]
pub struct File {
    id: u16,
    file_path: Option<String>,
    pages: u16,
    format: KavitaFileType,
    bytes: u32,
    created: NaiveTime,
    extension: Option<String>,
}
