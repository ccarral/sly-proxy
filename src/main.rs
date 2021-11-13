mod config;
mod dispatch;
mod display;
mod error;
mod listener;
mod proxy;
mod target;
use crate::config::AppConfig;
use crate::dispatch::DispatchService;
use crate::display::display_app;
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

    let targets = ["127.0.0.1:8080", "127.0.0.1:8081", "127.0.0.1:8082"]
        .into_iter()
        .map(|addr| {
            let addr = addr.parse::<SocketAddr>().unwrap();
            Target::from_sock_addr(&addr)
        })
        .collect::<Vec<Target>>();

    let config = AppConfig {
        listen_on: vec![8083, 8084],
        name: "pepe-app".into(),
        targets,
    };

    let app = app_builder(config);

    runtime.block_on(async move {
        let (a,) = tokio::join!(app);
        a?;
        Ok::<(), SlyError>(())
    })?;

    Ok(())
}

pub async fn app_builder(config: AppConfig) -> Result<(), SlyError> {
    display_app(&config);
    let (tx, rx) = mpsc::channel(100);
    let listener_handles = {
        config
            .ports()
            .iter()
            .map(|port| ListenerService::new(tx.clone()).on_port(*port))
            .map(|listener_svc| tokio::spawn(listener_svc.run()))
    };

    let listeners_futures = futures::future::try_join_all(listener_handles);

    // Needs to run on a tokio task so .accept() will be polled
    let listener_task = tokio::spawn(listeners_futures);

    let dispatch_handle = {
        let dispatch_service = DispatchService::new(rx).with_targets(config.targets);
        dispatch_service.run()
    };

    let (listener_result, dispatch_result) = tokio::join!(listener_task, dispatch_handle);

    // Not very neat
    listener_result.map_err(|e| SlyError::Generic(format!("Unable to join threads: {}", e)))??;
    dispatch_result?;

    Ok(())
}
