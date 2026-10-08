use super::KavitaClient;
use crate::models::library::KavitaLibrary;

impl KavitaClient {
    pub async fn list_libraries(&self) -> Result<Vec<KavitaLibrary>, reqwest::Error> {
        self.execute_json(
            self.http_client
                .get(format!("{}/api/Library/libraries", self.base_url)),
        )
        .await
    }

    pub async fn library(&self, id: u16) -> Result<KavitaLibrary, reqwest::Error> {
        self.execute_json(
            self.http_client
                .get(format!("{}/api/Library?libraryId={}", self.base_url, id)),
        )
        .await
    }

    pub async fn user_libraries(&self, user_id: u32) -> Result<Vec<KavitaLibrary>, reqwest::Error> {
        self.execute_json(self.http_client.get(format!(
            "{}/api/Library/user-libraries?userId={}",
            self.base_url, user_id
        )))
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn client_for(mock_server: &MockServer) -> KavitaClient {
        KavitaClient::new(mock_server.uri(), Client::new())
    }

    const LIBRARY_JSON: &str = r#"{
        "id": 5,
        "name": "Comics",
        "type": 1,
        "coverImage": null,
        "includeInDashboard": true,
        "includeInRecomended": true,
        "includeInSearch": true,
        "manageReadingLists": false,
        "manageCollections": false,
        "libraryFileTypes": [1, 2],
        "folders": ["/data/comics"]
    }"#;

    #[tokio::test]
    async fn list_libraries_parses_response() {
        let mock_server = MockServer::start().await;
        let body = format!("[{}]", LIBRARY_JSON);

        Mock::given(method("GET"))
            .and(path("/api/Library/libraries"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .mount(&mock_server)
            .await;

        let client = client_for(&mock_server);
        let libraries = client.list_libraries().await.unwrap();

        let expected: KavitaLibrary = serde_json::from_str(LIBRARY_JSON).unwrap();
        assert_eq!(libraries, vec![expected]);
    }

    #[tokio::test]
    async fn library_parses_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/Library"))
            .and(query_param("libraryId", "5"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(LIBRARY_JSON, "application/json"))
            .mount(&mock_server)
            .await;

        let client = client_for(&mock_server);
        let library = client.library(5).await.unwrap();

        let expected: KavitaLibrary = serde_json::from_str(LIBRARY_JSON).unwrap();
        assert_eq!(library, expected);
    }

    #[tokio::test]
    async fn user_libraries_parses_response() {
        let mock_server = MockServer::start().await;
        let body = format!("[{}]", LIBRARY_JSON);

        Mock::given(method("GET"))
            .and(path("/api/Library/user-libraries"))
            .and(query_param("userId", "7"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .mount(&mock_server)
            .await;

        let client = client_for(&mock_server);
        let libraries = client.user_libraries(7).await.unwrap();

        let expected: KavitaLibrary = serde_json::from_str(LIBRARY_JSON).unwrap();
        assert_eq!(libraries, vec![expected]);
    }

    #[tokio::test]
    async fn library_errors_on_malformed_body() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/Library"))
            .respond_with(ResponseTemplate::new(200).set_body_raw("not json", "application/json"))
            .mount(&mock_server)
            .await;

        let client = client_for(&mock_server);
        let result = client.library(5).await;

        assert!(result.is_err());
    }
}
