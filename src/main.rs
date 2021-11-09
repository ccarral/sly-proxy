mod dispatch;
mod error;
mod listener;
mod proxy;
use proxy::TcpProxy;
use std::error::Error;
use std::io;
use tokio::net::TcpListener;
use tokio::runtime::Builder;
use tower::Service;
use tracing_subscriber::EnvFilter;

fn main() -> Result<(), Box<dyn Error>> {
    // Create a runtime

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("Initializing runtime");
    let runtime = Builder::new_current_thread().enable_all().build()?;

    runtime.block_on(async {
        let source_addr = "127.0.0.1:11";

        tracing::info!("Binding to {:?}", &source_addr);
        let source = TcpListener::bind(source_addr).await?;

        let destination = "127.0.0.1:8081".parse().unwrap();

        match source.accept().await {
            Ok((stream, addr)) => {
                tracing::info!("Connected to {:?}", addr);
                let mut proxy_service = TcpProxy::new(destination);
                proxy_service.call(stream).await?;
            }
            Err(e) => {
                tracing::error!("Encountered error while accepting socket: {}", e);
            }
        }
        // match source.accept().await =>{};
        Ok::<(), io::Error>(())
    })?;

    Ok(())
}
