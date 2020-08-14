use std::fmt::Display;
use actix::{Context, Handler, Actor, ResponseFuture};
use actix_derive::{Message, MessageResponse};
use ttl_cache::TtlCache;
use crate::gcs::GetObjectResult;
use std::time::Duration;
use std::{pin::Pin, future::Future, convert::Infallible};
use actix::prelude::*;
use std::io;

#[derive(MessageResponse, Debug, Clone)]
pub struct CacheEntry {
    body: Vec<u8>
}

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

pub struct CachingActor {
    cache: TtlCache<String, CacheEntry>,
    ttl: Duration,
}

impl CachingActor {
    pub fn new(capacity: Option<usize>, ttl: Option<u64>) -> Self {
        Self {
            cache: TtlCache::new(capacity.unwrap_or(100)),
            ttl: Duration::from_secs(ttl.unwrap_or(3600))
        }
    }

    async fn get_key(&mut self, msg: GetCacheEntry) -> Option<CacheEntry> {
        self.cache.get(&msg.key).map(|v| v.clone())
    }
}

impl Actor for CachingActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("local cache actor is alive");
    }
}

impl Handler<PutCacheEntry> for CachingActor {
    type Result = ();

    fn handle(&mut self, msg: PutCacheEntry, _: &mut Context<Self>) -> Self::Result {
        self.cache.insert(
            msg.key.into(),
            CacheEntry {
                body: msg.body,
            },
            self.ttl.clone(),
        );
    }
}

#[derive(Debug)]
pub struct MyStruct {}

impl Handler<GetCacheEntry> for CachingActor {
    type Result = ResponseFuture<Result<u32, io::Error>>;

    fn handle(&mut self, msg: GetCacheEntry, _: &mut Context<Self>) -> Self::Result {
        Box::pin(async move {
            Ok(32 as u32)
        })
    }
}
