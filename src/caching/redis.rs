use actix::{Context, Handler, Actor, ResponseFuture};
use crate::caching::messages::{CacheEntry, GetCacheEntry, PutCacheEntry, CacheError};
use redis_async::resp_array;
use tokio::sync::Mutex;
use std::sync::Arc;
use prometheus::{Counter, register_counter};

const KEY_PREFIX: &'static str = "cloud_storage_proxy";

lazy_static! {
    static ref REDIS_CACHE_GET: Counter = register_counter!(
        "redis_cache_get",
        "redis cache get operations"
    ).unwrap();
    static ref REDIS_CACHE_PUT: Counter = register_counter!(
        "redis_cache_put",
        "redis cache put operations"
    ).unwrap();
}

pub struct RedisCache {
    client: Arc<Mutex<redis_async::client::PairedConnection>>,
    ttl: u64,
}

impl RedisCache {
    pub async fn new(host: String, port: u16, ttl: Option<u64>) -> Result<Self, CacheError> {
        let address = format!("{}:{}", &host, &port)
            .parse()
            .map_err(|source| CacheError::FailedToCreateCacheClient { reason: format!("failed to parse address: {}", source) })?;

        let client = Arc::new(Mutex::new(redis_async::client::paired_connect(&address).await
            .map_err(|_| CacheError::FailedToCreateCacheClient { reason: "Failed to create redis client".to_string() })?));

        Ok(Self {
            client,
            ttl: ttl.unwrap_or(3600)
        })
    }
}

impl Actor for RedisCache {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
    }
}

impl Handler<PutCacheEntry> for RedisCache {
    type Result = ResponseFuture<Result<(), CacheError>>;

    fn handle(&mut self, msg: PutCacheEntry, _: &mut Context<Self>) -> Self::Result {
        let client = self.client.clone();
        let msg = msg.clone();
        let ttl = self.ttl.clone();

        REDIS_CACHE_PUT.inc();

        Box::pin(async move {
            let client = client.lock().await;

            let key = format!("{}:{}:{}", KEY_PREFIX, msg.bucket, msg.key);
            let entry = serde_json::to_string(&msg.entry)?;
            client.send_and_forget(resp_array!["SET", &key, entry]);
            client.send_and_forget(resp_array!["EXPIRE", &key, format!("{}", &ttl)]);

            Ok(())
        })
    }
}

impl Handler<GetCacheEntry> for RedisCache {
    type Result = ResponseFuture<Result<CacheEntry, CacheError>>;

    fn handle(&mut self, msg: GetCacheEntry, _: &mut Context<Self>) -> Self::Result {
        let client = self.client.clone();
        let msg = msg.clone();

        REDIS_CACHE_GET.inc();

        Box::pin(async move {
            let client = client.lock().await;

            let key = format!("{}:{}:{}", KEY_PREFIX, msg.bucket, msg.key);
            let entry_str = client.send::<String>(resp_array!["GET", key]).await
                .map_err(|err| CacheError::FailedToGetKey { reason: format!("{}", err) })?;
            let entry = serde_json::from_str(&entry_str)?;

            Ok(entry)
        })
    }
}
