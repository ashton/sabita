use crate::models::library_item::LibraryItem;

pub mod kavita;

pub trait Provider {
    fn name(&self) -> String;
    async fn list_items(&self) -> Vec<LibraryItem>;
}
