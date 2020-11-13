use std::collections::HashMap;
use actix::{Actor, Context, Handler};
use crate::rate_limiting::messages::{PutRateLimitingStats, RateLimitingEntry};

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
    type Result = Result<(), RateLitmitingError>;

    fn handle(&mut self, msg: RateLimitingEntry, _: &mut Context<Self>) -> Self::Result {
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
