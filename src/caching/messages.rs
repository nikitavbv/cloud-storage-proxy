use std::io;
use actix_derive::{Message, MessageResponse};

#[derive(Message)]
#[rtype(result = "()")]
pub struct PutCacheEntry {
    pub bucket: String,
    pub key: String,
    pub body: Vec<u8>
}

#[derive(Message)]
#[rtype(result = "io::Result<u32>")]
pub struct GetCacheEntry {
    pub bucket: String,
    pub key: String
}
