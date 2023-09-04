use hyper::{
    service::{make_service_fn, service_fn},
    Server,
};
use rhfs::{config, server};
use std::sync::Arc;
use std::{convert::Infallible, net::SocketAddr};

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let cfg = config::get_config();
    println!("参数:{:?}", cfg);
    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.port));
    let handle_request = Arc::new(server::handle_request);
    let make_svc = make_service_fn(|_conn| {
        let handle_request = Arc::clone(&handle_request);
        async { Ok::<_, Infallible>(service_fn(move |req| handle_request(req))) }
    });
    let server = Server::bind(&addr).serve(make_svc);
    println!("Listening on http://{}", addr);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
