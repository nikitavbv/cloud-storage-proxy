#[macro_use] extern crate serde_derive;
#[macro_use] extern crate lazy_static;
extern crate custom_error;
#[macro_use] extern crate log;
extern crate ttl_cache;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Request, Body, Response, Server, Method, Error, StatusCode, header::{HeaderValue, HeaderName}};
use crate::config::{load_config, Config};
use crate::gcs::{GoogleCloudStorageClient, GCSClientError};
use std::fs;
use std::{sync::Arc, env::var};
use gcs::GetObjectResult;
use crate::caching::messages::{GetCacheEntry, PutCacheEntry, CacheEntry};
use crate::caching::caching::Caching;
use config::BucketConfiguration;
use tokio::sync::Mutex;
use std::net::SocketAddr;
use std::collections::HashMap;
use prometheus::{TextEncoder, Encoder, Counter, register_counter};

mod config;
mod gcs;
mod caching;

lazy_static! {
    static ref REQUEST_OK_COUNTER: Counter = register_counter!(
        "request_ok",
        "requests successfully processed"
    ).unwrap();
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let config = Arc::new(load_config()?);
    let addr = config.ip_addr().unwrap_or([0, 0, 0, 0].into());
    let addr = SocketAddr::new(addr, config.port.unwrap_or(8080));
    let cache: Arc<Caching> = Arc::new(Caching::new(config.caching.as_ref().unwrap_or(&HashMap::new())).await);
    let client = Arc::new(Mutex::new(GoogleCloudStorageClient::new(&service_account_key(&config)).await?));

    let make_svc = make_service_fn(move |_| {
        let config = config.clone();
        let client = client.clone();
        let cache = cache.clone();

        async move {
            Ok::<_, Error>(service_fn(move |_req| {
                let config = config.clone();
                let client = client.clone();
                let cache = cache.clone();

                async move { proxy_service(_req, &config, client.clone(), cache.clone()).await }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    Ok(())
}

async fn proxy_service(
    req: Request<Body>,
    config: &Config,
    gcs: Arc<Mutex<GoogleCloudStorageClient>>,
    cache: Arc<Caching>,
) -> Result<Response<Body>, String> {
    if req.method() != Method::GET {
        return Ok(Response::new("wrong method".into()));
    }

    if config.metrics.unwrap_or(false) || req.uri().path() == config.metrics_endpoint.as_ref().unwrap_or(&"/metrics".to_string()) {
        return Ok(response_for_metrics_endpoint());
    }

    let host = req.headers().get("Host").unwrap().to_str().unwrap();
    let bucket = config.bucket_configuration_by_host(&host).unwrap();
    let bucket_name = bucket.bucket.as_ref().unwrap().as_str();
    let mut object_name = req.uri().path().to_string();

    trace!("GET {} {}", bucket_name, object_name);

    if object_name.starts_with("/") {
        object_name = object_name[1..].into();
    }

    // TODO: handle dirs
    if object_name.is_empty() || object_name.ends_with("/") {
        object_name = format!(
            "{}{}",
            object_name,
            bucket.index.as_ref().unwrap_or(&"index.html".to_string()).clone()
        );
    }

    return Ok(if let Some(cache_name) = &bucket.cache_name {
        let cache = cache.get_cache(&cache_name);
        if let Some(cache) = cache {
            debug!("using cache");

            let get_from_cache_message = GetCacheEntry {
                bucket: bucket_name.to_string(),
                key: object_name.to_string()
            };

            let res = match cache.send_get_message(get_from_cache_message).await {
                Ok(v) => v,
                Err(err) => {
                    warn!("failed to get object from cache: {}", err);

                    let obj = match gcs.lock().await.get_object(bucket_name, &object_name).await {
                        Ok(v) => v,
                        Err(err) => return Ok(response_for_gcs_client_error(err, &bucket, &bucket_name, &object_name, gcs.clone()).await)
                    };

                    let entry = CacheEntry::from_body_and_headers(obj.body, obj.headers);

                    let put_cache_message = PutCacheEntry {
                        bucket: bucket_name.to_string(),
                        key: object_name.to_string(),
                        entry: entry.clone(),
                    };

                    if let Err(err) = cache.send_put_message(put_cache_message).await {
                        error!("failed to save gcs response to cache: {}", err);
                    }

                    entry
                }
            };

            REQUEST_OK_COUNTER.inc();
            response_for_object(&bucket, res.to_get_object_result())
        } else {
            debug!("cache instance not found");
            get_object_mapped_to_response(gcs.clone(), &bucket, bucket_name, &object_name).await
        }
    } else {
        debug!("skipping caching");
        get_object_mapped_to_response(gcs.clone(), &bucket, bucket_name, &object_name).await
    })
}

async fn get_object_mapped_to_response(gcs: Arc<Mutex<GoogleCloudStorageClient>>, bucket: &BucketConfiguration, bucket_name: &str, object_name: &str) -> Response<Body> {
    match gcs.lock().await.get_object(bucket_name, &object_name).await {
        Ok(v) => {
            REQUEST_OK_COUNTER.inc();
            response_for_object(&bucket, v)
        },
        Err(err) => response_for_gcs_client_error(err, &bucket, &bucket_name, &object_name, gcs.clone()).await
    }
}

async fn response_for_gcs_client_error(
    err: GCSClientError, 
    bucket: &BucketConfiguration, 
    bucket_name: &str, 
    _object_name: &str,
    gcs: Arc<Mutex<GoogleCloudStorageClient>>,
) -> Response<Body> {
    let is_not_found = match err {
        GCSClientError::ObjectNotFound => true,
        _ => false
    };

    if is_not_found {
        let not_found_object_name = bucket.not_found.as_ref()
            .unwrap_or(&"404.html".to_string())
            .clone();

        return match gcs.lock().await.get_object(bucket_name, &not_found_object_name).await {
            Ok(v) => response_for_object(&bucket,v),
            Err(_) => match Response::builder()
                    .status(StatusCode::from_u16(404).unwrap())
                    .body("not found.".into()) {
                Ok(v) => v,
                Err(err) => {
                    error!("failed to create response: {}", err);
                    return Response::new("internal server error".into());
                }
            }
        };
    }

    error!("failed to get gcs object: {}", err);

    match Response::builder()
        .status(StatusCode::from_u16(500).unwrap())
        .body("failed to get gcs object".into()) {
        Ok(v) => v,
        Err(err) => {
            error!("failed to create response: {}", err);
            Response::new("internal server error".into())
        }
    }
}

fn response_for_object(config: &BucketConfiguration, object: GetObjectResult) -> Response<Body> {
    let mut res = Response::builder()
        .status(StatusCode::from_u16(200).unwrap())
        .body(object.body.into()).unwrap();

    let headers = res.headers_mut();

    for (k, v) in object.headers {
        headers.insert(HeaderName::from_lowercase(k.as_bytes()).unwrap(), HeaderValue::from_str(&v).unwrap());
    }

    if let Some(headers_to_append) = &config.headers {
        for (k, v) in headers_to_append {
            headers.insert(HeaderName::from_lowercase(k.as_bytes()).unwrap(), HeaderValue::from_str(&v).unwrap());
        }
    }

    return res;
}

fn response_for_metrics_endpoint() -> Response<Body> {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();

    if let Err(err) = encoder.encode(&metric_families, &mut buffer) {
        error!("failed to write metrics: {}", err);
        return Response::new("failed to write metrics".into());
    }

    let encoded = match String::from_utf8(buffer) {
        Ok(v) => v,
        Err(err) => {
            error!("failed to encode metrics: {}", err);
            return Response::new("failed to encode metrics".into());
        }
    };

    Response::builder().status(StatusCode::from_u16(200).unwrap()).body(encoded.into()).unwrap()
}

fn service_account_key(config: &Config) -> String {
    match &config.service_account_key {
        Some(v) => v.to_string(),
        None => fs::read_to_string(
            &config.clone().service_account_key_file.unwrap_or(get_service_account_key_file_name())
        ).expect("failed to read service account key file")
    }
}

fn get_service_account_key_file_name() -> String {
    var("SERVICE_ACCOUNT_KEY_FILE").unwrap_or("service_account_key.json".into())
}