use std::fmt::Display;
use actix::{Context, Handler, Actor, ResponseFuture};
use ttl_cache::TtlCache;
use crate::gcs::GetObjectResult;
use std::time::Duration;
use std::{pin::Pin, future::Future, convert::Infallible};
use actix::prelude::*;
use std::io;
use crate::caching::messages::{CacheEntry, GetCacheEntry, PutCacheEntry};

custom_error!{pub RedisCacheError
    FailedToCreateRedisClient = "failed to create redis client"
}

pub struct RedisCache {
    client: redis_async::client::PairedConnection,
    ttl: u64,
}

impl RedisCache {
    pub fn new(ttl: Option<u64>) -> Result<Self, RedisCacheError> {
        let address = format!("{}:{}", ip_addr, redis_port())
            .parse()
            .map_err(|source| RedisClientError::FailedToParseAddress { source })?;

        let client    async fn get_key(&mut self, msg: GetCacheEntry) -> Option<CacheEntry> {
        match self.cache.get(&msg.key) {
            Some(v) => Some(v.clone()),
            None => None
        }
    } = redis_async::client::paired_connect(&address).await
            .map_err(|_| RedisClientError::FailedToCreateRedisClient)?;

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
    type Result = ();

    fn handle(&mut self, msg: PutCacheEntry, _: &mut Context<Self>) -> Self::Result {
        // TODO: implement this
    }
}

impl Handler<GetCacheEntry> for Redis {
    type Result = ResponseFuture<Result<CacheEntry, io::Error>>;

    fn handle(&mut self, msg: GetCacheEntry, _: &mut Context<Self>) -> Self::Result {
        // TODO: implement this
    }
}
