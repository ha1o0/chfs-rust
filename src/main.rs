use hyper::{
    service::{make_service_fn, service_fn},
    Server,
};
use log::LevelFilter;
use rhfs::{config, server::handle_request};
use std::{
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cfg = config::get_config();
    println!("参数:{:?}", cfg);
    env_logger::builder()
        .filter_level(LevelFilter::from_str(&cfg.log).unwrap())
        .init();
    let addr_ipv4 = (Ipv4Addr::new(0, 0, 0, 0), cfg.port).into();
    let addr_ipv6 = (Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), cfg.port).into();
    let make_svc = make_service_fn(|_conn| {
        // 这里可以添加中间件或其他逻辑
        async { Ok::<_, hyper::Error>(service_fn(handle_request)) }
    });
    let server_ipv4 = Server::bind(&addr_ipv4).serve(make_svc);
    let server_ipv6 = Server::bind(&addr_ipv6).serve(make_svc);
    println!("Listening on http://{}, http://{}", addr_ipv4, addr_ipv6);
    let (ipv4, ipv6) = tokio::join!(server_ipv4, server_ipv6);
    ipv4?;
    ipv6?;
    Ok(())
    // For hyper 1.0
    // let addr_v6 = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, cfg.port, 0, 0);
    // let listener_v6 = TcpListener::bind(addr_v6).await?;
    // loop {
    //     let (stream_v6, _) = listener_v6.accept().await?;
    //     serve_connection(stream_v6).await;
    // }
}

// For hyper 1.0
// async fn serve_connection(stream: TcpStream) {
//     // Spawn a tokio task to serve multiple connections concurrently
//     tokio::task::spawn(async move {
//         // Use an adapter to access something implementing `tokio::io` traits as if they implement
//         // `hyper::rt` IO traits.
//         let io = TokioIo::new(stream);
//         // Finally, we bind the incoming connection to our `hello` service
//         if let Err(err) = http1::Builder::new()
//             .keep_alive(true)
//             // `service_fn` converts our function in a `Service`
//             .serve_connection(io, service_fn(handle_request))
//             .await
//         {
//             log::error!("Error serving connection: {:?}", err);
//         }
//     });
// }
