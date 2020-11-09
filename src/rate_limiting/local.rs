pub struct LocalRateLimiter {
    counter: TtlCache<String, u64>,
    ttl: Duration,
}

impl LocalRateLimiter {
    pub fn new(capacity: Option<usize>, ttl: Option<u64>) -> Self {
        Self {
            cache: TtlCache::new(capacity.unwrap_or(100)),
            ttl: Duration::from_secs(ttl.unwrap_or(3600))
        }
    }
}

impl Actor for LocalRateLimiter {
    type Context = Context<Self>;
}
