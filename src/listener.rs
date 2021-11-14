use crate::error::SlyError;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;
use tokio::{net::TcpStream, sync::mpsc};

pub struct ListenerService {
    port: u16,
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
        self.port = port;
        self
    }

    /// Returns a future that runs the listener task
    pub async fn run(self) -> Result<(), SlyError> {
        let tx = self.tx;
        tracing::info!("building listener service on port {}.", self.port);
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), self.port);
        let listener = TcpListener::bind(addr).await.unwrap();
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    tracing::info!("connection accepted from {:?}", &addr);
                    tx.send(stream).await?;
                    Ok(())
                }
                Err(e) => Err(e),
            }?
        }
    }
}
