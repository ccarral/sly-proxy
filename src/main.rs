mod dispatch;
mod error;
mod listener;
mod proxy;
use proxy::TcpProxy;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::runtime::Builder;
use tokio::sync::mpsc;
use tower::Service;
use tracing_subscriber::EnvFilter;

fn main() -> Result<(), Box<dyn Error>> {
    // Create a runtime

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("Initializing runtime");
    let runtime = Builder::new_current_thread().enable_all().build()?;

    let (tx, rx) = mpsc::channel(10);

    // Spawn service that listens on a multitude of ports and sends received streams through the tz
    // channel
    let listener_service = listener::ListenerService::new(tx)
        .on_port(11)
        .on_port(8081)
        .start(&runtime);

    let targets: Vec<SocketAddr> = [
        "127.0.0.1:9000",
        "127.0.0.1:9001",
        "127.0.0.1:9002",
        "127.0.0.1:9003",
    ]
    .into_iter()
    .map(|addr| addr.parse::<SocketAddr>())
    .collect::<Result<Vec<SocketAddr>, _>>()?;

    let dispatch_service = dispatch::DispatchService::new(rx)
        .with_targets(targets)
        .build(&runtime);

    runtime.block_on(async move {
        // dispatch_service.await?;
        tokio::select! {
            res = dispatch_service =>{
                match res{
                    Ok(_) => {},
                    Err(e) => {eprintln!("Unexpected error on dispatcher: {}",e);},
                }
            },
            _ = listener_service =>{
            }
        };
    });

    Ok(())
}
