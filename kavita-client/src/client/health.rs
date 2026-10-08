use tracing::debug;

use crate::client::KavitaClient;

impl KavitaClient {
    /// Hits Kavita's health check endpoint, returning an error if the server
    /// is unreachable or responds with a non-success status.
    pub async fn health(&self) -> Result<(), reqwest::Error> {
        let response = self
            .execute(
                self.http_client
                    .get(format!("{}/api/Health", self.base_url)),
            )
            .await?;

        if let Err(error) = response.error_for_status() {
            debug!("health check returned an error status: {error}");
            return Err(error);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn client_for(mock_server: &MockServer) -> KavitaClient {
        KavitaClient::new(mock_server.uri(), Client::new())
    }

    #[tokio::test]
    async fn health_succeeds_on_success_status() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/Health"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let client = client_for(&mock_server);

        assert!(client.health().await.is_ok());
    }

    #[tokio::test]
    async fn health_errors_on_failure_status() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/Health"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&mock_server)
            .await;

        let client = client_for(&mock_server);

        assert!(client.health().await.is_err());
    }
}
