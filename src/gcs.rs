use custom_error::custom_error;
use yup_oauth2::{ServiceAccountAuthenticator, read_service_account_key, ServiceAccountKey};
use yup_oauth2::authenticator::Authenticator;

custom_error!{GCSClientError
    FailedToReadAccountKey{source: std::error::Error} = "failed to read service account key: {source}",
    FailedToAuthToServiceAccount{source: std::io::Error} = "failed to auth to service account: {source}"
}

#[derive(Clone)]
pub struct GoogleCloudStorageClient {
    authenticator: Authenticator<i64>,
    reqwest_client: reqwest::Client
}

impl GoogleCloudStorageClient {

    pub async fn new(service_account_key: &str) -> Result<Self, GCSClientError> {
        let service_account_key: ServiceAccountKey = serde_json::from_str(service_account_key)
            .map_err(|source| GCSClientError::FailedToReadAccountKey { source })?;

        let authenticator = ServiceAccountAuthenticator::builder(service_account_key)
            .build().await.map_err(|source| GCSClientError::FailedToAuthToServiceAccount { source })?;

        GoogleCloudStorageClient {
            authenticator,
            reqwest_client: reqwest::Client::new(),
        }
    }
}