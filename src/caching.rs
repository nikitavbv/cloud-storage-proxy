use crate::gcs::GetObjectResult;
use ttl_cache::TtlCache;
use std::time::Duration;

pub trait GCSObjectCache {
    fn put(&mut self, bucket_name: &str, object_name: &str, object: GetObjectResult);
    fn get(&self, bucket_name: &str, object_name: &str) -> Option<&GetObjectResult>;
}

pub struct LocalCache {
    cache: TtlCache<(String, String), GetObjectResult>,
}

impl LocalCache {
    pub fn new(capacity: usize) -> Self {
        LocalCache {
            cache: TtlCache::new(capacity)
        }
    }
}

impl GCSObjectCache for LocalCache {
    fn put(&mut self, bucket_name: &str, object_name: &str, object: GetObjectResult) {
        self.cache.insert(
            (bucket_name.into(), object_name.into()),
            object,
            Duration::from_secs(60 * 60)
        );
    }

    fn get(&self, bucket_name: &str, object_name: &str) -> Option<&GetObjectResult> {
        self.cache.get(&(bucket_name.into(), object_name.into()))
    }
}