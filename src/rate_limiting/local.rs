use std::collections::HashMap;
use actix::{Actor, Context, Handler};
use crate::caching::messages::PutRateLimitingStats;

pub struct LocalRateLimiter {
    stats: HashMap<String, (u64, u64)>,
}

impl LocalRateLimiter {
    pub fn new() -> Self {
        Self {
            stats: HashMap::new()
        }
    }
}

impl Actor for LocalRateLimiter {
    type Context = Context<Self>;
}

impl Handler<PutRateLimitingStats> for LocalRateLimiter {
    type Result = Result<(), CacheError>;

    fn handle(&mut self, msg: PutCacheEntry, _: &mut Context<Self>) -> Self::Result {
        LOCAL_CACHE_SIZE.set(self.cache.iter().count() as f64);

        LOCAL_CACHE_PUT.inc();

        self.cache.insert(
            msg.key.into(),
            msg.entry,
            self.ttl.clone(),
        );
        Ok(())
    }
}
