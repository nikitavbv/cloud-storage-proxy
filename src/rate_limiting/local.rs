use std::collections::HashMap;
use actix::{Actor, Context};

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
