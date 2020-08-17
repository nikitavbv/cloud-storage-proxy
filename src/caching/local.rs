use std::fmt::Display;
use actix::{Context, Handler, Actor, ResponseFuture};
use ttl_cache::TtlCache;
use crate::gcs::GetObjectResult;
use std::time::Duration;
use std::{pin::Pin, future::Future, convert::Infallible};
use actix::prelude::*;
use std::io;
use crate::caching::messages::{CacheEntry, GetCacheEntry, PutCacheEntry};

pub struct LocalCache {
    cache: TtlCache<String, CacheEntry>,
    ttl: Duration,
}

impl LocalCache {
    pub fn new(capacity: Option<usize>, ttl: Option<u64>) -> Self {
        Self {
            cache: TtlCache::new(capacity.unwrap_or(100)),
            ttl: Duration::from_secs(ttl.unwrap_or(3600))
        }
    }

    async fn get_key(&mut self, msg: GetCacheEntry) -> Option<CacheEntry> {
        match self.cache.get(&msg.key) {
            Some(v) => Some(v.clone()),
            None => None
        }
    }
}

impl Actor for LocalCache {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("local cache actor is alive");
    }
}

impl Handler<PutCacheEntry> for LocalCache {
    type Result = ();

    fn handle(&mut self, msg: PutCacheEntry, _: &mut Context<Self>) -> Self::Result {
        self.cache.insert(
            msg.key.into(),
            msg.entry,
            self.ttl.clone(),
        );
    }
}

impl Handler<GetCacheEntry> for LocalCache {
    type Result = ResponseFuture<Result<CacheEntry, io::Error>>;

    fn handle(&mut self, msg: GetCacheEntry, _: &mut Context<Self>) -> Self::Result {
        Box::pin(async move {
            Ok(CacheEntry::new())
        })
    }
}
