use crate::gcs::GetObjectResult;
use ttl_cache::TtlCache;
use std::time::Duration;

pub trait GCSObjectCache {
    fn put(&mut self, object_name: &str, object: GetObjectResult);
    fn get(&self, object_name: &str) -> Option<&GetObjectResult>;
}

pub struct LocalCache {
    cache: TtlCache<String, GetObjectResult>,
}

impl LocalCache {
    pub fn new(capacity: usize) -> Self {
        LocalCache {
            cache: TtlCache::new(capacity)
        }
    }
}

impl GCSObjectCache for LocalCache {
    fn put(&mut self, object_name: &str, object: GetObjectResult) {
        self.cache.insert(
            object_name.into(),
            object,
            Duration::from_secs(60 * 60)
        );
    }

    fn get(&self, object_name: &str) -> Option<&GetObjectResult> {
        self.cache.get(object_name.into())
    }
}