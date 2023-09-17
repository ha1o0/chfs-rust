use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use log::LevelFilter;
use rhfs::{config, server::handle_request};
use std::{
    net::{Ipv6Addr, SocketAddrV6},
    str::FromStr,
};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Hello, world!");
    let cfg = config::get_config();
    println!("参数:{:?}", cfg);
    env_logger::builder()
        .filter_level(LevelFilter::from_str(&cfg.log).unwrap())
        .init();
    let addr_v6 = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, cfg.port, 0, 0);
    let listener_v6 = TcpListener::bind(addr_v6).await?;

    // We start a loop to continuously accept incoming connections
    loop {
        let (stream_v6, _) = listener_v6.accept().await?;
        serve_connection(stream_v6).await;
    }
}

async fn serve_connection(stream: TcpStream) {
    // Spawn a tokio task to serve multiple connections concurrently
    tokio::task::spawn(async move {
        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);
        // Finally, we bind the incoming connection to our `hello` service
        if let Err(err) = http1::Builder::new()
            .keep_alive(true)
            // `service_fn` converts our function in a `Service`
            .serve_connection(io, service_fn(handle_request))
            .await
        {
            log::error!("Error serving connection: {:?}", err);
        }
    });
}
