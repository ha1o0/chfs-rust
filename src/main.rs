use std::{convert::Infallible, net::SocketAddr};
use std::sync::Arc;
use hyper::{Body, Request, Response, Server, Method, service::{service_fn, make_service_fn}};

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let resp;
    match req.method() {
        &Method::GET => {
            resp = Response::new(Body::from("Hello, Webdav, GET"));
        },
        &Method::PUT => {
            resp = Response::new(Body::from("Hello, Webdav, PUT"));
        },
        _ => {
            resp = Response::new(Body::from("Hello, Webdav"));
        }
    }
    Ok(resp)
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    let handle_request = Arc::new(handle_request);
    let make_svc = make_service_fn(|_conn| {
        let handle_request = Arc::clone(&handle_request);
        async {
            Ok::<_, Infallible>(service_fn(move |req| {
                handle_request(req)
            }))
        }
    });
    let server = Server::bind(&addr).serve(make_svc);
    println!("Listening on http://{}", addr);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
