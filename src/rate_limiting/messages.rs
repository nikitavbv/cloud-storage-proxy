use actix_derive::Message;
use serde::{Serialize, Deserialize};
use custom_error::custom_error;

custom_error! {pub RateLimitingError
    FailedToCreateRateLimiterClient {reason: String} = "failed to create rate limiter client: {}",
    SerdeError {source: serde_json::Error} = "failed to serialize/deserialize entry: {source}",
    FailedToGetKey {reason: String} = "failed to get key: {reason}",
    FailedToSendMessage {source: actix::MailboxError} = "failed to send message: {source}"
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