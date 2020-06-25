#[derive(Clone)]
pub struct GoogleCloudStorageClient {
    service_account_key: String,
    reqwest_client: reqwest::Client
}

impl GoogleCloudStorageClient {

    pub fn new(service_account_key: &str) -> Self {
        GoogleCloudStorageClient {
            service_account_key: service_account_key.into(),
            reqwest_client: reqwest::Client::new(),
        }
    }
}