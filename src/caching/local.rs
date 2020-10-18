use actix::{Context, Handler, Actor, ResponseFuture};
use ttl_cache::TtlCache;
use std::time::Duration;

use prometheus::{Gauge, register_gauge};

use crate::caching::messages::{CacheEntry, GetCacheEntry, PutCacheEntry, CacheError};

lazy_static! {
    static ref LOCAL_CACHE_SIZE: Gauge = register_gauge!(
        "local_cache_size",
        "local cache size"
    ).unwrap();
}

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
}

impl Actor for LocalCache {
    type Context = Context<Self>;
}

impl Handler<PutCacheEntry> for LocalCache {
    type Result = Result<(), CacheError>;

    fn handle(&mut self, msg: PutCacheEntry, _: &mut Context<Self>) -> Self::Result {
        self.cache.insert(
            msg.key.into(),
            msg.entry,
            self.ttl.clone(),
        );
        Ok(())
    }
}

impl Handler<GetCacheEntry> for LocalCache {
    type Result = ResponseFuture<Result<CacheEntry, CacheError>>;

    fn handle(&mut self, msg: GetCacheEntry, _: &mut Context<Self>) -> Self::Result {
        let key = msg.key.clone();
        let cache = self.cache.clone();

        Box::pin(async move {
            cache.get(&key).map(|v| v.clone()).ok_or(CacheError::FailedToGetKey {
                reason: "Key not present".to_string()
            })
        })
    }
}
