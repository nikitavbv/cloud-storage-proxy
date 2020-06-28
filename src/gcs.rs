use custom_error::custom_error;
use yup_oauth2::{ServiceAccountAuthenticator, read_service_account_key, ServiceAccountKey};
use yup_oauth2::authenticator::{Authenticator, DefaultHyperClient, HyperClientBuilder};
use std::io::ErrorKind;
use std::sync::Arc;

custom_error!{pub GCSClientError
    FailedToReadAccountKey{details: String} = "failed to read service account key: {details}",
    FailedToAuthToServiceAccount{source: std::io::Error} = "failed to auth to service account: {source}"
}

impl From<GCSClientError> for std::io::Error {

    fn from(err: GCSClientError) -> Self {
        std::io::Error::new(ErrorKind::Other, format!("gcs client error: {}", err))
    }
}

#[derive(Clone)]
pub struct GoogleCloudStorageClient {
    authenticator: Arc<Authenticator<<DefaultHyperClient as HyperClientBuilder>::Connector>>,
    reqwest_client: reqwest::Client
}

impl GoogleCloudStorageClient {

    pub async fn new(service_account_key: &str) -> Result<Self, GCSClientError> {
        let service_account_key: ServiceAccountKey = serde_json::from_str(service_account_key)
            .map_err(|source| GCSClientError::FailedToReadAccountKey { details: format!("{}", source) })?;

        let authenticator = ServiceAccountAuthenticator::builder(service_account_key)
            .build().await.map_err(|source| GCSClientError::FailedToAuthToServiceAccount { source })?;

        Ok(GoogleCloudStorageClient {
            authenticator: Arc::new(authenticator),
            reqwest_client: reqwest::Client::new(),
        })
    }

    pub async fn get_object(&self, bucket_name: &str, object: &str) -> Result<GetObjectResult, GCSClientError> {
        let access_token = &self.authenticator.token(
            vec!["https://www.googleapis.com/auth/devstorage.full_control"]).await?;

        let res = self.reqwest_client.get(&format!(
            "https://www.googleapis.com/storage/v1/b/{}/o/{}",
            bucket_name,
            object
        ));

        Ok(GetObjectResult::new(res)?)
    }
}