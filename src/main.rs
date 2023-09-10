use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use log::LevelFilter;
use rhfs::{config, server::handle_request};
use std::{net::SocketAddr, str::FromStr, sync::Arc};
use tokio::{net::TcpListener, sync::Semaphore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Hello, world!");
    let cfg = config::get_config();
    println!("参数:{:?}", cfg);
    env_logger::builder()
        .filter_level(LevelFilter::from_str(&cfg.log).unwrap())
        .init();
    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.port));
    let keep_alive_timeout = 60;

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;
    let concurrent_connections = 3;
    let semaphore = Arc::new(Semaphore::new(concurrent_connections));
    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;
        // Clone semaphore for each connection
        let semaphore_clone = semaphore.clone();

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Acquire a permit from the semaphore
            let permit = semaphore_clone.acquire().await;
            log::info!("permit: {:?}", permit);
            // Use an adapter to access something implementing `tokio::io` traits as if they implement
            // `hyper::rt` IO traits.
            let io = TokioIo::new(stream);
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                .keep_alive(Some(keep_alive_timeout).is_some())
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(handle_request))
                .await
            {
                log::error!("Error serving connection: {:?}", err);
            }
            drop(permit);
            log::info!("drop");
        });
    }
}
