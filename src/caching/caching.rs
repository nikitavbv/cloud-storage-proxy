use async_trait::async_trait;

use crate::gcs::GetObjectResult;
use ttl_cache::TtlCache;
use std::{thread, time::{self, Duration}};
use actix::{Actor, Context, Message, Handler};

#[async_trait]
pub trait GCSObjectCache {
    async fn put(&mut self, object_name: &str, object: GetObjectResult);
    async fn get(&self, object_name: &str) -> Option<GetObjectResult>;
}

pub struct NoCaching {
}

impl NoCaching {
    pub fn new() -> Self {
        NoCaching {
        }
    }
}

#[async_trait]
impl GCSObjectCache for NoCaching {
    async fn put(&mut self, _object_name: &str, _object: GetObjectResult) {
        // do nothing
    }

    async fn get(&self, _object_name: &str) -> Option<GetObjectResult> {
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

#[async_trait]
impl GCSObjectCache for LocalCache {
    async fn put(&mut self, object_name: &str, object: GetObjectResult) {
        self.cache.insert(
            object_name.into(),
            object,
            self.ttl.clone(),
        );
    }

    async fn get(&self, object_name: &str) -> Option<GetObjectResult> {
        self.cache.get(object_name.into()).map(|v| v.clone())
    }
}
