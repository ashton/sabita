use crate::{client::KavitaClient, models::chapter::Chapter};

impl KavitaClient {
    pub async fn chapter_detail(&self, chapter_id: u16) -> Result<Chapter, reqwest::Error> {
        self.http_client
            .get(format!(
                "{}/api/Chapter?chapterId={}",
                self.base_url, chapter_id
            ))
            .send()
            .await?
            .json()
            .await
    }
}
