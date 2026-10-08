use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    database,
    models::library::{Library, LibraryType},
};

pub async fn all() -> Result<Vec<Library>, String> {
    use crate::schema::libraries::dsl::*;

    let mut conn = database::connect().await.map_err(|e| e.to_string())?;
    libraries
        .select(Library::as_select())
        .load(&mut conn)
        .await
        .map_err(|e| e.to_string())
}

pub async fn find_by_external_id(remote_id: &str) -> Result<Library, String> {
    use crate::schema::libraries::dsl::*;

    let mut conn = database::connect().await.map_err(|e| e.to_string())?;
    libraries
        .filter(external_id.eq(remote_id))
        .select(Library::as_select())
        .first(&mut conn)
        .await
        .map_err(|e| e.to_string())
}

pub async fn create_from_folder(folder: String, name: String) -> Result<Library, String> {
    use crate::schema::libraries::dsl::libraries;

    let new_library = Library {
        id: Uuid::new_v4().to_string(),
        kind: LibraryType::Comic,
        name,
        external_id: None,
        folder: Some(folder),
        cover: None,
        integration_id: None,
    };

    let mut conn = database::connect().await.map_err(|e| e.to_string())?;
    diesel::insert_into(libraries)
        .values(&new_library)
        .execute(&mut conn)
        .await
        .map_err(|e| e.to_string())?;

    Ok(new_library)
}

pub async fn create(library: Library) -> Result<Library, String> {
    use crate::schema::libraries::dsl::libraries;

    let mut conn = database::connect().await.map_err(|e| e.to_string())?;
    diesel::insert_into(libraries)
        .values(&library)
        .execute(&mut conn)
        .await
        .map_err(|e| e.to_string())?;

    Ok(library)
}
