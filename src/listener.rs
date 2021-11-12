use crate::dispatch::DispatchService;
use crate::error::FlyError;
use futures::lock::Mutex;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use tokio::{net::TcpStream, sync::mpsc};

pub struct ListenerService {
    port: Vec<u16>,
    tx: mpsc::Sender<TcpStream>,
}

impl ListenerService {
    pub fn new(tx: mpsc::Sender<TcpStream>) -> Self {
        ListenerService {
            port: Default::default(),
            tx,
        }
    }

    pub fn on_port(mut self, port: u16) -> ListenerService {
        if !self.port.contains(&port) {
            self.port.push(port);
        }
        self
    }

    /// Spawn a task that listens on a port and blocks until an `.accept()` is received and sends
    /// the TcpStream through a channel
    fn listener_builder_inner(
        addr: SocketAddr,
        tx: mpsc::Sender<TcpStream>,
    ) -> JoinHandle<Result<(), FlyError<TcpStream>>> {
        tokio::spawn(async move {
            tracing::info!("Binding listener on {:?}", &addr);
            let listener = TcpListener::bind(addr).await?;
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    tracing::debug!("Connection accepted from {:?}", &addr);
                    // let dispatch_svc = DispatchService::
                    tx.send(stream).await?;
                    Ok(())
                }
                Err(e) => Err(e),
            }?;

            tracing::debug!("Listener service for {:?} ended succesfully", &addr);

            Ok(())
        })
    }

    /// Returns a future that runs the listener tasks
    pub async fn run(self) {
        let tx = self.tx;
        tracing::info!("Building listener svc.");
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), self.port[0]);
        let listener = TcpListener::bind(addr).await.unwrap();
        while let Ok((stream, addr)) = listener.accept().await {
            tracing::info!("Connection accepted from {:?}", &addr);
            tx.send(stream).await.unwrap();
            // Ok(())
        }
    }
}
