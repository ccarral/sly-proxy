mod dispatch;
mod error;
mod listener;
mod proxy;
use crate::dispatch::DispatchService;
use crate::error::SlyError;
use crate::listener::ListenerService;
use std::error::Error;
use std::net::SocketAddr;
use tokio::runtime::Builder;
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("Initializing runtime");

    // Create a runtime
    let runtime = Builder::new_multi_thread().enable_all().build()?;

    let app = app_builder();

    runtime.block_on(app)?;

    Ok(())
}

pub async fn app_builder() -> Result<(), SlyError> {
    let (tx, rx) = mpsc::channel(100);
    let listener_handle = {
        let listener_service = ListenerService::new(tx).on_port(8083);
        listener_service.run()
    };

    // Needs to run on a tokio task so .accept() will be polled
    let listener_task = Box::pin(tokio::spawn(listener_handle));

    let dispatch_handle = Box::pin({
        let targets: Vec<SocketAddr> = ["127.0.0.1:8080", "127.0.0.1:8081", "127.0.0.1:8082"]
            .into_iter()
            .map(|addr| addr.parse::<SocketAddr>())
            .collect::<Result<Vec<SocketAddr>, _>>()
            .unwrap();

        let dispatch_service = DispatchService::new(rx).with_targets(targets);
        dispatch_service.run()
    });

    let (a, b) = tokio::join!(listener_task, dispatch_handle);

    a.map_err(|e| SlyError::Generic(format!("Unable to join threads: {}", e)))??;
    b?;

    Ok(())
}
