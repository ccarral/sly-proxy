use crate::error::FlyError;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio::{net::TcpStream, sync::mpsc};

/// Builds a ListenerService
struct ListenerServiceBuilder {
    ports: Vec<u16>,
    tx: Option<mpsc::Sender<(TcpStream, SocketAddr)>>,
}

impl ListenerServiceBuilder {
    pub fn new() -> Self {
        ListenerServiceBuilder {
            ports: Default::default(),
            tx: None,
        }
    }
    pub fn with_port(&mut self, port: u16) -> &mut Self {
        if !self.ports.contains(&port) {
            self.ports.push(port);
        }
        self
    }

    pub fn with_tx(&mut self, tx: mpsc::Sender<(TcpStream, SocketAddr)>) -> &mut Self {
        self.tx = Some(tx);
        self
    }

    /// Consumes self and yields a task that runs listeners asynchronously
    pub fn build_service(self) -> Result<ListenerService, FlyError<(TcpStream, SocketAddr)>> {
        if let Some(tx) = self.tx {
            Ok(ListenerService {
                ports: self.ports,
                tx,
            })
        } else {
            Err(FlyError::Generic("Unable to build listener".into()))
        }
    }
}

struct ListenerService {
    ports: Vec<u16>,
    tx: mpsc::Sender<(TcpStream, SocketAddr)>,
}

impl ListenerService {
    const CHANNEL_BUFFER: usize = 10;

    /// Spawn a task that listens on a port and blocks until an `.accept()` is received and sends
    /// the TcpStream through a channel
    fn listener_builder_inner(
        addr: SocketAddr,
        tx: mpsc::Sender<(TcpStream, SocketAddr)>,
    ) -> JoinHandle<Result<(), FlyError<(TcpStream, SocketAddr)>>> {
        tokio::spawn(async move {
            let listener = TcpListener::bind(addr).await?;
            match listener.accept().await {
                Ok((stream, addr)) => {
                    tx.send((stream, addr)).await?;
                    Ok(())
                }
                Err(e) => Err(e),
            }?;

            Ok(())
        })
    }

    pub fn start(
        self,
    ) -> futures::future::JoinAll<
        JoinHandle<JoinHandle<Result<(), FlyError<(TcpStream, SocketAddr)>>>>,
    > {
        let listener_handles = self.ports.iter().map(|port| {
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), *port);
            let tx = self.tx.clone();
            tokio::spawn(async move { Self::listener_builder_inner(addr, tx) })
        });

        futures::future::join_all(listener_handles)
    }
}
