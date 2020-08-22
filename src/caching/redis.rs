use std::fmt::Display;
use actix::{Context, Handler, Actor, ResponseFuture};
use ttl_cache::TtlCache;
use crate::gcs::GetObjectResult;
use std::time::Duration;
use std::{pin::Pin, future::Future, convert::Infallible};
use actix::prelude::*;
use std::io;
use crate::caching::messages::{CacheEntry, GetCacheEntry, PutCacheEntry};
use custom_error::custom_error;
use std::net::AddrParseError;

custom_error!{pub RedisCacheError
    FailedToParseAddress {source: AddrParseError} = "failed to parse address: {source}",
    FailedToSerialize {source: serde_json::Error} = "failed to serialize entry: {source}",
    FailedToCreateRedisClient = "failed to create redis client"
}

const KEY_PREFIX: &'static str = "cloud_storage_proxy";

pub struct RedisCache {
    client: redis_async::client::PairedConnection,
    ttl: u64,
}

impl RedisCache {
    pub async fn new(host: String, port: u16, ttl: Option<u64>) -> Result<Self, RedisCacheError> {
        let address = format!("{}:{}", &host, &port)
            .parse()
            .map_err(|source| RedisCacheError::FailedToParseAddress { source })?;

        let client = redis_async::client::paired_connect(&address).await
            .map_err(|_| RedisCacheError::FailedToCreateRedisClient)?;

        Ok(Self {
            client,
            ttl: ttl.unwrap_or(3600)
        })
    }
}

impl Actor for RedisCache {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("local cache actor is alive");
    }
}

impl Handler<PutCacheEntry> for RedisCache {
    type Result = Result<(), RedisCacheError>;

    fn handle(&mut self, msg: PutCacheEntry, _: &mut Context<Self>) -> Self::Result {
        let key = format!("{}:{}:{}", KEY_PREFIX, msg.bucket, msg.key);
        let entry = serde_json::to_string(&msg.entry)?;
        connection.send_and_forget(resp_array!["SET", key, entry]);
        connection.send_and_forget(resp_array!["EXPIRE", key, format!("{}", &self.ttl)]);
    }
}

impl Handler<GetCacheEntry> for RedisCache {
    type Result = ResponseFuture<Result<CacheEntry, io::Error>>;

    fn handle(&mut self, msg: GetCacheEntry, _: &mut Context<Self>) -> Self::Result {
        // TODO: implement this
    }
}
