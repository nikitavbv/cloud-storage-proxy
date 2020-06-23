#[macro_use] extern crate serde_derive;
extern crate custom_error;

use hyper::service::{make_service_fn, service_fn};
use std::convert::Infallible;
use hyper::{Request, Body, Response, Server, Method, Error};
use crate::config::{load_config, Config};
use std::sync::Arc;
use std::ops::Deref;
use std::borrow::Borrow;

mod config;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let addr = ([127, 0, 0, 1], 8080).into();

    let config = Arc::new(load_config()?);

    let make_svc = make_service_fn(move |_| {
        let config = config.clone();

        async move {
            Ok::<_, Error>(service_fn(move |_req| {
                let config = config.clone();

                async move { proxy_service(_req, &config).await }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    Ok(())
}

async fn proxy_service(req: Request<Body>, config: &Config) -> Result<Response<Body>, Infallible> {
    if req.method() != Method::GET {
        return Ok(Response::new("wrong method".into()));
    }

    println!("host is {}", req.headers().get("Host").unwrap().to_str().unwrap());
    Ok(Response::new("hello".into()))
}