use std::collections::HashMap;
use actix::{Actor, Context, Handler};
use crate::rate_limiting::messages::{PutRateLimitingStats, RateLimitingEntry, RateLimitingError};

lazy_static! {
    static ref LOCAL_RATE_LIMITED: Gauge = register_gauge!(
        "local_rate_limiter_limited",
        "local rate limiter limited requests"
    ).unwrap();
    static ref LOCAL_OK: Counter = register_counter!(
        "local_rate_limiter_ok",
        "local rate limiter ok requests"
    ).unwrap();
}

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
