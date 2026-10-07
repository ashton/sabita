use crate::schema::servers;
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;

#[derive(Debug, DbEnum)]
pub enum ServerType {
    Kavita,
    Suwayomi,
    Opds,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = servers)]
pub struct ServerConfiguration {
    name: String,
    server_type: ServerType,
    url: String,
}
