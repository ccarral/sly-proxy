mod dispatch;
mod error;
mod listener;
mod proxy;
mod target;
use crate::dispatch::DispatchService;
use crate::error::SlyError;
use crate::listener::ListenerService;
use std::error::Error;
use std::net::SocketAddr;
use target::Target;
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

    let ports = [8083, 8084];

    let targets1 = ["127.0.0.1:8080", "127.0.0.1:8081", "127.0.0.1:8082"]
        .into_iter()
        .map(|addr| {
            let addr = addr.parse::<SocketAddr>().unwrap();
            Target(addr)
        });

    let app = app_builder(targets1, ports);

    runtime.block_on(async move {
        let (a,) = tokio::join!(app);
        a?;
        Ok::<(), SlyError>(())
    })?;

    Ok(())
}

pub async fn app_builder<T, P>(targets: T, ports: P) -> Result<(), SlyError>
where
    T: IntoIterator<Item = Target>,
    P: IntoIterator<Item = u16>,
{
    let (tx, rx) = mpsc::channel(100);
    let listener_handles = {
        ports
            .into_iter()
            .map(|port| ListenerService::new(tx.clone()).on_port(port))
            .map(|listener_svc| tokio::spawn(listener_svc.run()))
    };

    let listeners_futures = futures::future::try_join_all(listener_handles);

    // Needs to run on a tokio task so .accept() will be polled
    let listener_task = tokio::spawn(listeners_futures);

    let dispatch_handle = {
        let dispatch_service = DispatchService::new(rx).with_targets(targets);
        dispatch_service.run()
    };

    let (a, b) = tokio::join!(listener_task, dispatch_handle);

    // Not very neat
    a.map_err(|e| SlyError::Generic(format!("Unable to join threads: {}", e)))??;
    b?;

    Ok(())
}
