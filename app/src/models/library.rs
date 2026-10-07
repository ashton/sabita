use diesel::{Insertable, Selectable, deserialize::Queryable};

use crate::schema::libraries;

#[derive(Clone, Debug, Queryable, Selectable, Insertable, PartialEq)]
#[diesel(table_name = libraries)]
pub struct Library {
    pub id: String,
    pub name: String,
    pub external_id: Option<String>,
    pub folder: Option<String>,
    pub cover: Option<String>,
}
