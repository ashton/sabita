use kavita_client::models::series::Series as KavitaSeries;

use crate::{adapter::ItemAdapter, models::library_item::Item};

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
