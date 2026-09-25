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
}
