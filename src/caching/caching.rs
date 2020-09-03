use custom_error::custom_error;
use std::collections::HashMap;
use crate::caching::local::LocalCache;
use crate::caching::redis::RedisCache;
use crate::config;
use actix::Addr;

custom_error!{pub CacheInstantiationError
    MissingField { field_name: String } = "missing field: {field_name}",
    NotImplemented { cache_type: String } = "cache not implemented: {cache_type}"
}

pub struct Caching {
    caches: HashMap<String, CacheInstance>
}

impl Caching {
    pub async fn new(config: &HashMap<String, Caching>) -> Self {
        let mut caches = HashMap::new();

        for cache_config in config {
            let cache = match make_cache(cache_config.1).await {
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
            Some(v) => match v {
                "local" => Ok(LocalCache::new(&config.capacity.unwrap()).start()),
                cache_type => Err(CacheInstantiationError::NotImplemeted { cache_type: cache_type.clone() })
            },
            None => Err(CacheInstantiationError::MissingField { field_name: "caching_type".to_string() })
        }
    }
}

pub enum CacheInstance {
    LocalCache(Addr<LocalCache>),
    Redis(Addr<RedisCache>),
}
