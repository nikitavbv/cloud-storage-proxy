#[macro_use] extern crate serde_derive;
extern crate custom_error;
#[macro_use] extern crate log;
extern crate ttl_cache;

use hyper::service::{make_service_fn, service_fn};
use std::convert::Infallible;
use hyper::{Request, Body, Response, Server, Method, Error, StatusCode, header::{HeaderValue, HeaderName}};
use crate::config::{load_config, Config};
use crate::gcs::{GoogleCloudStorageClient, GCSClientError};
use std::fs;
use std::{sync::Arc, env::var, collections::HashMap};
use gcs::GetObjectResult;
use caching::{GCSObjectCache, LocalCache};
use config::{Caching, BucketConfiguration};
use tokio::sync::Mutex;
use crate::caching::NoCaching;

mod config;
mod gcs;
mod caching;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let addr = ([0, 0, 0, 0], 8080).into();

    let config = Arc::new(load_config()?);
    let client = GoogleCloudStorageClient::new(&service_account_key(&config)).await?;
    let client = Arc::new(client);
    let cache = Arc::new(Mutex::new(HashMap::new()));

    let make_svc = make_service_fn(move |_| {
        let config = config.clone();
        let client = client.clone();
        let cache = cache.clone();

        async move {
            Ok::<_, Error>(service_fn(move |_req| {
                let config = config.clone();
                let client = client.clone();
                let cache = cache.clone();

                async move { proxy_service(_req, &config, &client, cache.clone()).await }
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
    gcs: &GoogleCloudStorageClient,
    cache: Arc<Mutex<HashMap<String, Box<dyn GCSObjectCache + Send>>>>,
) -> Result<Response<Body>, Infallible> {
    if req.method() != Method::GET {
        return Ok(Response::new("wrong method".into()));
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
    if object_name.is_empty() && object_name.ends_with("/") {
        object_name = format!(
            "{}{}",
            object_name,
            bucket.index.as_ref().unwrap_or(&"index.html".to_string()).clone()
        );
    }

    let mut cache = cache.lock().await;

    println!("here4");

    let mut cache = match cache.get_mut(bucket_name) {
        Some(v) => v,
        None => {
            let new_cache = make_cache(&match config.caching.clone() {
                Some(v) => match bucket.caching.clone() {
                    Some(v2) => v.override_with(&v2),
                    None => v,
                },
                None => bucket.caching.clone()
                    .expect("expected caching to be set for bucket, as no caching is set globally")
            });
            cache.insert(bucket_name.into(), new_cache);
            cache.get_mut(bucket_name).unwrap()
        }
    };

    let object = cache.get(&object_name);
    let object = match object.clone() {
        Some(v) => {
            println!("cache hit");
            v.clone()
        },
        None => {
            println!("cache miss");
            let obj = match gcs.get_object(bucket_name, &object_name).await {
                Ok(v) => v,
                Err(err) => return Ok(response_for_gcs_client_error(err, &bucket, &bucket_name, &object_name, &gcs).await)
            };
            cache.put(&object_name, obj.clone());
            obj
        }
    };

    Ok(response_for_object(object))
}

async fn response_for_gcs_client_error(
    err: GCSClientError, 
    bucket: &BucketConfiguration, 
    bucket_name: &str, 
    object_name: &str, 
    gcs: &GoogleCloudStorageClient,
) -> Response<Body> {
    let is_not_found = match err {
        GCSClientError::ObjectNotFound => true,
        _ => false
    };

    if is_not_found {
        let not_found_object_name = bucket.not_found.as_ref()
            .unwrap_or(&"404.html".to_string())
            .clone();
        
        return match gcs.get_object(bucket_name, &not_found_object_name).await {
            Ok(v) => response_for_object(v),
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

fn response_for_object(object: GetObjectResult) -> Response<Body> {
    let mut res = Response::builder()
        .status(StatusCode::from_u16(200).unwrap())
        .body(object.body.into()).unwrap();

    let headers = res.headers_mut();
    for (k, v) in object.headers {
        headers.insert(HeaderName::from_lowercase(k.as_bytes()).unwrap(), HeaderValue::from_str(&v).unwrap());
    }

    return res;
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

fn make_cache(caching: &Caching) -> Box<dyn GCSObjectCache + Send> {
    let caching_type= &caching.caching_type.as_ref().unwrap()[..];

    match caching_type {
        "local" => Box::new(LocalCache::new(caching.capacity.unwrap())),
        _ => Box::new(NoCaching::new()),
    }
}