use diesel::{Insertable, Selectable, deserialize::Queryable};
use diesel_derive_enum::DbEnum;

use crate::schema::libraries;

#[derive(Clone, Debug, DbEnum, PartialEq)]
pub enum LibraryType {
    Manga,
    Comic,
    Ebook,
}

impl LibraryType {
    pub fn label(&self) -> &'static str {
        match self {
            LibraryType::Manga => "Manga",
            LibraryType::Comic => "Comic",
            LibraryType::Ebook => "Ebook",
        }
    }
}

#[derive(Clone, Debug, Queryable, Selectable, Insertable, PartialEq)]
#[diesel(table_name = libraries)]
pub struct Library {
    pub id: String,
    pub kind: LibraryType,
    pub name: String,
    pub external_id: Option<String>,
    pub folder: Option<String>,
    pub cover: Option<String>,
    pub integration_id: Option<String>,
}
