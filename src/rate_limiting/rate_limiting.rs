use std::collections::HashMap;
use super::messages::RateLimitingError;
use crate::config;
use custom_error::custom_error;
use crate::rate_limiting::local::LocalRateLimiter;
use actix::{Addr, Actor};

custom_error!{pub RateLimiterInstantiationError
    MissingField { field_name: String } = "missing field: {field_name}",
    NotImplemented { rate_limiter_type: String } = "rate limiter not implemented: {rate_limiter_type}",
    RateLimiterError { source: RateLimitingError } = "rate limiter error: {}"
}

pub struct RateLimiting {
    rate_limiters: HashMap<String, RateLimiterInstance>
}

impl RateLimiting {
    pub async fn new(config: &HashMap<String, config::RateLimitingConfiguration>) -> Self {
        let mut rate_limiters = HashMap::new();

        for rate_limiter_config in config {
            let rate_limiter = match Self::make_rate_limiter(rate_limiter_config.1).await {
                Ok(v) => v,
                Err(err) => {
                    error!("failed to make rate_limite: {}", err);
                    continue;
                }
            };
            rate_limiters.insert(rate_limiter_config.0.clone(), rate_limiter);
        }

        Self {
            rate_limiters
        }
    }

    async fn make_rate_limiter(config: &config::RateLimitingConfiguration) -> Result<RateLimiterInstance, RateLimiterInstantiationError> {
        match &config.rate_limiting_type {
            Some(v) => match &v as &str {
                "local" => Ok(RateLimiterInstance::LocalRateLimiter(LocalRateLimiter::new().start())),
                cache_type => Err(RateLimiterInstantiationError::NotImplemented { rate_limiter_type: v.to_string() })
            },
            None => Err(RateLimiterInstantiationError::MissingField { field_name: "rate_limiter_type".to_string() })
        }
    }

    pub fn get_rate_limiter(&self, name: &str) -> Option<RateLimiterInstance> {
        (&self.rate_limiters.get(name).map(|v| v.clone())).clone()
    }
}

#[derive(Clone)]
pub enum RateLimiterInstance {
    LocalRateLimiter(Addr<LocalRateLimiter>),
    //Redis(Addr<RedisRateLimiter>),
}
