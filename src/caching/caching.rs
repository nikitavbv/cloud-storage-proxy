use custom_error::custom_error;
use std::collections::HashMap;
use crate::caching::local::LocalCache;
use crate::caching::redis::RedisCache;
use crate::caching::messages::{CacheError, PutCacheEntry, GetCacheEntry, CacheEntry};
use crate::config;
use actix::{Actor, Addr};

custom_error!{pub CacheInstantiationError
    MissingField { field_name: String } = "missing field: {field_name}",
    NotImplemented { cache_type: String } = "cache not implemented: {cache_type}",
    CacheError { source: CacheError } = "cache error: {}"
}

pub struct Caching {
    caches: HashMap<String, CacheInstance>
}

impl Caching {
    pub async fn new(config: &HashMap<String, config::Caching>) -> Self {
        let mut caches = HashMap::new();

        for cache_config in config {
            let cache = match Self::make_cache(cache_config.1).await {
                Ok(v) => v,
                Err(err) => {
                    error!("failed to make cache: {}", err);
                    continue;
                }
            };
            caches.insert(cache_config.0.clone(), cache);
        }

        Caching {
            caches
        }
    }

    async fn make_cache(config: &config::Caching) -> Result<CacheInstance, CacheInstantiationError> {
        match &config.caching_type {
            Some(v) => match &v as &str {
                "local" => Ok(CacheInstance::LocalCache(LocalCache::new(config.capacity, config.ttl).start())),
                "redis" => Ok(CacheInstance::Redis(
                    RedisCache::new(config.host.clone().unwrap(), config.port.unwrap(), config.ttl).await?.start()
                )),
                cache_type => Err(CacheInstantiationError::NotImplemented { cache_type: cache_type.to_string() })
            },
            None => Err(CacheInstantiationError::MissingField { field_name: "caching_type".to_string() })
        }
    }

    pub fn get_cache(&self, name: &str) -> Option<CacheInstance> {
        (&self.caches.get(name).map(|v| v.clone())).clone()
    }
}

#[derive(Clone)]
pub enum CacheInstance {
    LocalCache(Addr<LocalCache>),
    Redis(Addr<RedisCache>),
}

impl CacheInstance {

    pub async fn send_put_message(&self, msg: PutCacheEntry) -> Result<(), CacheError> {
        match &self {
            Self::LocalCache(addr) => addr.send(msg).await?,
            Self::Redis(addr) => addr.send(msg).await?
        }
    }

    pub async fn send_get_message(&self, msg: GetCacheEntry) -> Result<CacheEntry, CacheError> {
        match &self {
            Self::LocalCache(addr) => addr.send(msg).await?,
            Self::Redis(addr) => addr.send(msg).await?
        }
    }
}