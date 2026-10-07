use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    database,
    models::integration::{Integration, IntegrationType},
};

pub async fn all() -> Result<Vec<Integration>, String> {
    use crate::schema::integrations::dsl::*;

    let mut conn = database::connect().await.map_err(|e| e.to_string())?;
    integrations
        .select(Integration::as_select())
        .load(&mut conn)
        .await
        .map_err(|e| e.to_string())
}

pub async fn create(
    name: String,
    integration_type: IntegrationType,
    url: Option<String>,
    api_key: Option<String>,
) -> Result<Integration, String> {
    use crate::schema::integrations::dsl::integrations;

    let new_integration = Integration {
        id: Uuid::new_v4().to_string(),
        name,
        integration_type,
        url,
        api_key,
    };

    let mut conn = database::connect().await.map_err(|e| e.to_string())?;
    diesel::insert_into(integrations)
        .values(&new_integration)
        .execute(&mut conn)
        .await
        .map_err(|e| e.to_string())?;

    Ok(new_integration)
}
