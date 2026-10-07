use crate::{client::KavitaClient, models::user::User};

impl KavitaClient {
    pub async fn user_data(&self) -> Result<User, reqwest::Error> {
        self.http_client
            .get(format!("{}/api/Account", self.base_url))
            .send()
            .await?
            .json()
            .await
    }

    /// Exchanges a Kavita API key for a JWT, to be used as a bearer token on
    /// subsequent requests.
    pub async fn authenticate(&self, api_key: &str, plugin_name: &str) -> Result<User, reqwest::Error> {
        self.http_client
            .post(format!(
                "{}/api/Plugin/authenticate?apiKey={}&pluginName={}",
                self.base_url, api_key, plugin_name
            ))
            .send()
            .await?
            .json()
            .await
    }
}
