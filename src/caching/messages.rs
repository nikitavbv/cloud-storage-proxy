use std::io;
use actix_derive::{Message, MessageResponse};
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
    headers: HashMap<string, string>
}

impl CacheEntry {

    pub fn new() -> Self {
        CacheEntry {
            body: Vec::new(),
            headers: HashMap::new(),
        }
    }

    pub fn from_body_and_headers(body: Vec<u8>, headers: HashMap<string, string>) -> Self {
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