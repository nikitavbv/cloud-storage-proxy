#[derive(Clone)]
pub struct GoogleCloudStorageClient {
    reqwest_client: reqwest::Client
}

impl GoogleCloudStorageClient {

    pub fn new() -> Self {
        GoogleCloudStorageClient {
            reqwest_client: reqwest::Client::new(),
        }
    }

}