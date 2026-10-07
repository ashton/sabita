use crate::models::library_item::Item;

pub mod kavita;

pub trait Provider {
    fn name(&self) -> String;
    async fn list_items(&self) -> Vec<Item>;
}
