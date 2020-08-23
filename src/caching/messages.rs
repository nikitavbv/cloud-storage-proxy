use std::io;
use actix_derive::{Message, MessageResponse};
use serde::{Serialize, Deserialize};
use custom_error::custom_error;

custom_error! {pub CacheError
    FailedToCreateCacheClient {source: String} = "failed to create cache client: {}",
    SerdeError {source: serde_json::Error} = "failed to serialize/deserialize entry: {source}"
}

#[derive(Message)]
#[rtype(result = "Result<(), CacheError>")]
pub struct PutCacheEntry {
    pub bucket: String,
    pub key: String,
    pub entry: CacheEntry
}

#[derive(Message)]
#[rtype(result = "Result<CacheEntry, CacheError>")]
pub struct GetCacheEntry {
    pub bucket: String,
    pub key: String
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    body: Vec<u8>
}

impl CacheEntry {

    pub fn new() -> Self {
        CacheEntry {
            body: Vec::new()
        }
    }

    pub fn from_body(body: Vec<u8>) -> Self {
        CacheEntry {
            body
        }
    }
}