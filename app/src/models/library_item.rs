#[derive(Clone, Debug, PartialEq)]
pub struct LibraryItem {
    pub name: String,
    pub cover: String,
    pub library_id: String,
    pub pages: Option<u16>,
}
