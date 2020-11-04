use actix_derive::Message;
use serde::{Serialize, Deserialize};
use custom_error::custom_error;
use crate::gcs::GetObjectResult;
use std::collections::HashMap;

custom_error! {pub CacheError
    FailedToCreateCacheClient {reason: String} = "failed to create cache client: {}",
    SerdeError {source: serde_json::Error} = "failed to serialize/deserialize entry: {source}",
    FailedToGetKey {reason: String} = "failed to get key: {reason}",
    FailedToSendMessage {source: actix::MailboxError} = "failed to send message: {source}"
}

custom_error! {pub RateLimitingError
    FailedToCreateRateLimiterClient {reason: String} = "failed to create rate limiter client: {}",
    SerdeError {source: serde_json::Error} = "failed to serialize/deserialize entry: {source}",
    FailedToGetKey {reason: String} = "failed to get key: {reason}",
    FailedToSendMessage {source: actix::MailboxError} = "failed to send message: {source}"
}

#[derive(Message, Clone)]
#[rtype(result = "Result<(), CacheError>")]
pub struct PutCacheEntry {
    pub bucket: String,
    pub key: String,
    pub entry: CacheEntry
}

#[derive(Message, Clone)]
#[rtype(result = "Result<CacheEntry, CacheError>")]
pub struct GetCacheEntry {
    pub bucket: String,
    pub key: String
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    body: Vec<u8>,
    headers: HashMap<String, String>
}

impl CacheEntry {

    pub fn from_body_and_headers(body: Vec<u8>, headers: HashMap<String, String>) -> Self {
        CacheEntry {
            body,
            headers,
        }
    }

    pub fn to_get_object_result(self) -> GetObjectResult {
        GetObjectResult {
            body: self.body,
            headers: self.headers,
        }
    }
}

#[derive(Message, Clone)]
#[rtype(result = "Result<(), RateLimitingError>")]
pub struct PutRateLimitingStats {
    pub bucket: String,
    pub client: String
}

#[derive(Message, Clone)]
#[rtype(result = "Result<RateLimitingEntry, RateLimitingError>")]
pub struct GetRateLimitingStats {
    pub bucket: String,
    pub client: String
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RateLimitingEntry {
    bucket: String,
    client: String,
    requests: u64
}