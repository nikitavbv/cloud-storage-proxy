#[macro_use] extern crate serde_derive;
extern crate custom_error;

use std::sync::Arc;
use std::ops::Deref;
use std::borrow::Borrow;
use hyper::service::{make_service_fn, service_fn};
use std::convert::Infallible;
use hyper::{Request, Body, Response, Server, Method, Error, StatusCode};
use crate::config::{load_config, Config};
use crate::gcs::GoogleCloudStorageClient;
use std::fs;
use std::env::var;

mod config;
mod gcs;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let addr = ([127, 0, 0, 1], 8080).into();

    let config = Arc::new(load_config()?);
    let client = GoogleCloudStorageClient::new(&service_account_key(&config)).await?;
    let client = Arc::new(client);

    let make_svc = make_service_fn(move |_| {
        let config = config.clone();
        let client = client.clone();

        async move {
            Ok::<_, Error>(service_fn(move |_req| {
                let config = config.clone();
                let client = client.clone();

                async move { proxy_service(_req, &config, &client).await }
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
    gcs: &GoogleCloudStorageClient
) -> Result<Response<Body>, Infallible> {
    if req.method() != Method::GET {
        return Ok(Response::new("wrong method".into()));
    }

    let host = req.headers().get("Host").unwrap().to_str().unwrap();
    let bucket = config.bucket_configuration_by_host(&host).unwrap();
    let bucket_name = bucket.bucket.as_ref().unwrap().as_str();
    let mut object_name = req.uri().path();
    if object_name.starts_with("/") {
        object_name = &object_name[1..];
    }

    println!("host is {}", host);
    println!("bucket is {}", bucket_name);

    let object = match gcs.get_object(bucket_name, object_name).await {
        Ok(v) => v,
        Err(err) => {
            eprintln!("failed to get gcs object: {}", err);

            let errors_response = match Response::builder()
                .status(StatusCode::from_u16(500).unwrap())
                .body("failed to get gcs object".into()) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("failed to create response: {}", err);
                    return Ok(Response::new("internal server error".into()));
                }
            };

            return Ok(errors_response)
        }
    };

    Ok(Response::new(object.body.into()))
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
