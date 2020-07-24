use crate::gcs::GetObjectResult;
use ttl_cache::TtlCache;
use std::time::Duration;

pub trait GCSObjectCache {
    fn put(&mut self, object_name: &str, object: GetObjectResult);
    fn get(&self, object_name: &str) -> Option<&GetObjectResult>;
}

pub struct NoCaching {
}

impl NoCaching {
    pub fn new() -> Self {
        NoCaching {
        }
    }
}

impl GCSObjectCache for NoCaching {
    fn put(&mut self, _object_name: &str, _object: GetObjectResult) {
        // do nothing
    }

    fn get(&self, _object_name: &str) -> Option<&GetObjectResult> {
        None
    }
}

pub struct LocalCache {
    cache: TtlCache<String, GetObjectResult>,
    ttl: Duration,
}

impl LocalCache {
    pub fn new(capacity: Option<usize>, ttl: Option<u64>) -> Self {
        LocalCache {
            cache: TtlCache::new(capacity.unwrap_or(100)),
            ttl: Duration::from_secs(ttl.unwrap_or(3600))
        }
    }
}

impl GCSObjectCache for LocalCache {
    fn put(&mut self, object_name: &str, object: GetObjectResult) {
        self.cache.insert(
            object_name.into(),
            object,
            self.ttl.clone(),
        );
    }

    fn get(&self, object_name: &str) -> Option<&GetObjectResult> {
        self.cache.get(object_name.into())
    }
}