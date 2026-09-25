use crate::models::file::KavitaFileType;
use serde::Deserialize;
use serde_repr::Deserialize_repr;

#[derive(Deserialize_repr, Debug, Default, PartialEq)]
#[repr(u8)]
pub enum KavitaLibraryType {
    Manga = 0,
    #[default]
    Comic = 1,
    Book = 2,
    Image = 3,
    LightNovel = 4,
    ComicVine = 5,
}

#[derive(Deserialize, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct KavitaLibrary {
    pub id: u16,
    pub name: String,
    #[serde(rename = "type")]
    pub library_type: KavitaLibraryType,
    pub cover_image: Option<String>,
    pub include_in_dashboard: bool,
    pub include_in_recomended: bool,
    pub include_in_search: bool,
    pub manage_reading_lists: bool,
    pub manage_collections: bool,
    pub library_file_types: Vec<KavitaFileType>,
    pub folders: Vec<String>,
}
