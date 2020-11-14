use std::collections::HashMap;
use actix::{Actor, Context, Handler};
use crate::rate_limiting::messages::{PutRateLimitingStats, RateLimitingEntry, RateLimitingError};

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
    type Result = Result<(), RateLimitingError>;

    fn handle(&mut self, msg: PutRateLimitingStats, _: &mut Context<Self>) -> Self::Result {
        // todo: implement this
        Ok(())
    }
}
