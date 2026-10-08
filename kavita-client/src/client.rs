use reqwest::{Client, RequestBuilder, Response};
use serde::de::DeserializeOwned;
use tracing::debug;

mod account;
mod chapter;
mod health;
mod library;
mod series;
mod volume;

pub struct KavitaClient {
    http_client: Client,
    base_url: String,
}

impl KavitaClient {
    pub fn new(base_url: String, http_client: Client) -> Self {
        KavitaClient {
            base_url,
            http_client,
        }
    }

    /// Sends `request`, logging the method, URL, response status and any
    /// error at debug level.
    async fn execute(&self, request: RequestBuilder) -> Result<Response, reqwest::Error> {
        let request = match request.build() {
            Ok(request) => request,
            Err(error) => {
                debug!("failed to build request: {error}");
                return Err(error);
            }
        };

        let method = request.method().clone();
        let url = request.url().clone();

        debug!("{method} {url}");

        let result = self.http_client.execute(request).await;

        match &result {
            Ok(response) => debug!("{method} {url} -> {}", response.status()),
            Err(error) => debug!("{method} {url} failed: {error}"),
        }

        result
    }

    /// Sends `request` and decodes the JSON response body into `T`, logging
    /// the method, URL, response status and any error (request or decode)
    /// at debug level.
    ///
    /// A non-2xx status is reported as an error without attempting to
    /// decode `T` from it — Kavita's error responses are a validation
    /// problem object, not the requested shape, so trying to decode them
    /// as `T` would just mask the real status error behind a confusing
    /// decode failure. The raw body is still logged so the actual
    /// validation message is visible.
    async fn execute_json<T: DeserializeOwned>(
        &self,
        request: RequestBuilder,
    ) -> Result<T, reqwest::Error> {
        let response = self.execute(request).await?;

        if let Err(error) = response.error_for_status_ref() {
            let body = response.text().await.unwrap_or_default();
            debug!("response body: {body}");
            return Err(error);
        }

        let body = response.json::<T>().await;

        if let Err(error) = &body {
            debug!("failed to decode response body: {error}");
        }

        body
    }
}
