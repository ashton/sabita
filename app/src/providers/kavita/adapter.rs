use kavita_client::models::{library::KavitaLibrary, series::Series as KavitaSeries};
use uuid::Uuid;

use crate::{
    adapter::{ItemAdapter, LibraryAdapter},
    models::{library::Library, library_item::Item},
};

pub struct KavitaItemAdapter;

impl ItemAdapter<KavitaSeries> for KavitaItemAdapter {
    fn adapt_item(source: KavitaSeries) -> Item {
        Item {
            name: source.name,
            library_id: source.library_id.to_string(),
            cover: source.cover_image,
        }
    }
}

pub struct KavitaLibraryAdapter;

impl LibraryAdapter<KavitaLibrary> for KavitaLibraryAdapter {
    fn adapt_library(source: KavitaLibrary) -> Library {
        Library {
            id: Uuid::new_v4().to_string(),
            name: source.name,
            external_id: Some(source.id.to_string()),
            folder: None,
            cover: None,
        }
    }
}
