use kavita_client::client::KavitaClient;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

use crate::{
    adapter::LibraryAdapter, models::library::Library,
    providers::kavita::adapter::KavitaLibraryAdapter,
};

pub struct KavitaProvider {
    client: KavitaClient,
}

impl KavitaProvider {
    /// Exchanges the given API key for a JWT and builds a `KavitaProvider`
    /// that authenticates all subsequent requests with it.
    pub async fn authenticate(url: String, api_key: String) -> Result<Self, String> {
        let unauthenticated_client = KavitaClient::new(url.clone(), reqwest::Client::new());
        let user = unauthenticated_client
            .authenticate(&api_key, "Sabita")
            .await
            .map_err(|e| e.to_string())?;
        let token = user
            .token
            .ok_or_else(|| "kavita did not return an auth token".to_string())?;

        let mut headers = HeaderMap::new();
        let value = HeaderValue::from_str(&format!("Bearer {token}")).map_err(|e| e.to_string())?;
        headers.insert(AUTHORIZATION, value);

        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| e.to_string())?;

        Ok(Self {
            client: KavitaClient::new(url, http_client),
        })
    }

    /// Checks whether a Kavita server at `url` is reachable, without
    /// requiring authentication.
    pub async fn ping(url: String) -> Result<(), String> {
        let client = KavitaClient::new(url, reqwest::Client::new());
        client.health().await.map_err(|e| e.to_string())
    }

    /// Checks whether `api_key` is valid for the Kavita server at `url`.
    pub async fn check_authentication(url: String, api_key: String) -> Result<(), String> {
        Self::authenticate(url, api_key).await.map(|_| ())
    }

    pub async fn list_libraries(&self, integration_id: &str) -> Result<Vec<Library>, String> {
        let libraries = self
            .client
            .list_libraries()
            .await
            .map_err(|e| e.to_string())?;

        Ok(libraries
            .into_iter()
            .map(|library| KavitaLibraryAdapter::adapt_library(library, integration_id))
            .collect())
    }
}
