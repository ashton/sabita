use reqwest::Client;

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
}
