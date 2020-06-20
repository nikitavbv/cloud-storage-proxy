use hyper::service::{make_service_fn, service_fn};
use std::convert::Infallible;
use hyper::{Request, Body, Response, Server, Method, Method::GET};

#[tokio::main]
async fn main() {
    let addr = ([127, 0, 0, 1], 8080).into();

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(proxy_service))
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn proxy_service(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    if req.method() != GET {
        return Ok(Response::new("wrong method".into()));
    }

    println!("host is {}", req.headers().get("Host").unwrap().to_str().unwrap());
    Ok(Response::new("hello".into()))
}