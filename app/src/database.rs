use std::env;

use diesel::{ConnectionResult, SqliteConnection};
use diesel_async::{AsyncConnection, sync_connection_wrapper::SyncConnectionWrapper};

pub async fn connect() -> ConnectionResult<SyncConnectionWrapper<SqliteConnection>> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
    SyncConnectionWrapper::<SqliteConnection>::establish(&database_url).await
}
