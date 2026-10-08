use crate::{client::KavitaClient, models::volume::Volume};

impl KavitaClient {
    pub async fn list_volumes_from_series(
        &self,
        series_id: u16,
    ) -> Result<Vec<Volume>, reqwest::Error> {
        self.execute_json(self.http_client.get(format!(
            "{}/api/Series/volumes?seriesId={}",
            self.base_url, series_id
        )))
        .await
    }

    pub async fn volume_detail(&self, volume_id: u16) -> Result<Volume, reqwest::Error> {
        self.execute_json(self.http_client.get(format!(
            "{}/api/Volume?volumeId={}",
            self.base_url, volume_id
        )))
        .await
    }
}
