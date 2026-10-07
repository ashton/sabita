use crate::models::library_item::Item;

pub trait ItemAdapter<T> {
    fn adapt_item(source: T) -> Item;
}
