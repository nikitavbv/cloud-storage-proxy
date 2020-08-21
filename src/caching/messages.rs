use std::io;
use actix_derive::{Message, MessageResponse};
use serde::{Serialize, Deserialize};

#[derive(Message)]
#[rtype(result = "()")]
pub struct PutCacheEntry {
    pub bucket: String,
    pub key: String,
    pub entry: CacheEntry
}

#[derive(Message)]
#[rtype(result = "io::Result<CacheEntry>")]
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