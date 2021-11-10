use crate::error::FlyError;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::task::JoinHandle;
use tokio::{net::TcpStream, sync::mpsc};

pub struct ListenerService {
    ports: Vec<u16>,
    tx: mpsc::Sender<TcpStream>,
}

impl ListenerService {
    pub fn new(tx: mpsc::Sender<TcpStream>) -> Self {
        ListenerService {
            ports: Default::default(),
            tx,
        }
    }

    pub fn on_port(mut self, port: u16) -> ListenerService {
        if !self.ports.contains(&port) {
            self.ports.push(port);
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
    pub fn start(
        self,
        runtime: &Runtime,
    ) -> futures::future::JoinAll<JoinHandle<JoinHandle<Result<(), FlyError<TcpStream>>>>> {
        let listener_handles = self.ports.iter().map(|port| {
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), *port);
            let tx = self.tx.clone();
            runtime.spawn(async move { Self::listener_builder_inner(addr, tx) })
        });

        futures::future::join_all(listener_handles)
    }
}
