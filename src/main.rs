mod dispatch;
mod error;
mod listener;
mod proxy;
use std::error::Error;
use std::net::SocketAddr;
use tokio::runtime::Builder;
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;

use crate::error::FlyError;

fn main() -> Result<(), Box<dyn Error>> {
    // Create a runtime

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("Initializing runtime");
    let runtime = Builder::new_multi_thread().enable_all().build()?;

    let (tx, rx) = mpsc::channel(10);

    // Spawn service that listens on a multitude of ports and sends received streams through the tz
    // channel

    runtime.block_on(async move {
        let listener_handle = {
            let listener_service = listener::ListenerService::new(tx).on_port(8083);
            listener_service.run()
        };

        // Needs to run on a tokio task so .accept() will be polled
        let listener_task = tokio::spawn(listener_handle);

        let dispatch_handle = {
            let targets: Vec<SocketAddr> = ["127.0.0.1:8080", "127.0.0.1:8081", "127.0.0.1:8082"]
                .into_iter()
                .map(|addr| addr.parse::<SocketAddr>())
                .collect::<Result<Vec<SocketAddr>, _>>()
                .unwrap();

            let dispatch_service = dispatch::DispatchService::new(rx).with_targets(targets);
            dispatch_service.run()
        };

        let (a, b) = tokio::join!(listener_task, dispatch_handle);

        a.map_err(|e| FlyError::Generic(format!("Unable to join threads: {}", e)))??;
        b?;

        Ok::<(), FlyError>(())
    })?;

    Ok(())
}
