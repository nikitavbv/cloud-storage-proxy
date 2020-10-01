use custom_error::custom_error;
use yup_oauth2::{ServiceAccountAuthenticator, ServiceAccountKey};
use yup_oauth2::authenticator::{Authenticator, DefaultHyperClient, HyperClientBuilder};
use std::io::ErrorKind;
use std::{collections::HashMap, sync::Arc};

custom_error!{pub GCSClientError
    FailedToReadAccountKey{details: String} = "failed to read service account key: {details}",
    FailedToAuthToServiceAccount{source: std::io::Error} = "failed to auth to service account: {source}",
    OAuthError{source: yup_oauth2::error::Error} = "oauth failed: {source}",
    RequestFailed{source: reqwest::Error} = "request failed: {source}",
    ObjectNotFound = "object not found"
}

impl From<GCSClientError> for std::io::Error {

    fn from(err: GCSClientError) -> Self {
        std::io::Error::new(ErrorKind::Other, format!("gcs client error: {}", err))
    }
}

#[derive(Clone)]
pub struct GetObjectResult {
    pub body: Vec<u8>,
    pub headers: HashMap<String, String>
}

impl GetObjectResult {
    async fn new(res: reqwest::Response) -> Result<Self, GCSClientError> {
        if res.status() == 404 {
            return Err(GCSClientError::ObjectNotFound)
        }

        let headers = &res.headers().clone();
        let headers = headers.into_iter()
            .map(|v| (v.0.clone().to_string(), v.1.to_str().unwrap_or("").to_string()))
            .collect::<HashMap<String, String>>().clone();

        let body = res.bytes().await?.to_vec();

        Ok(GetObjectResult {
            body,
            headers,
        })
    }
}

#[derive(Clone)]
pub struct GoogleCloudStorageClient {
    authenticator: Arc<Authenticator<<DefaultHyperClient as HyperClientBuilder>::Connector>>,
    reqwest_client: reqwest::Client
}

impl GoogleCloudStorageClient {

    pub async fn new(service_account_key: &str) -> Result<Self  , GCSClientError> {
        let service_account_key: ServiceAccountKey = serde_json::from_str(&service_account_key)
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
            &vec!["https://www.googleapis.com/auth/devstorage.full_control"]).await?;

        let url = format!(
            "https://c.storage.googleapis.com/{}",
            object
        );

        let res = self.reqwest_client.get(&url)
            .header("Authorization", format!("Bearer {}", access_token.as_str()))
            .header("Host", bucket_name)
            .send()
            .await?;


        Ok(GetObjectResult::new(res).await?)
    }
}