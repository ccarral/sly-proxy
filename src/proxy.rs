use std::error::Error;
use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::Poll;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpSocket;
use tokio::net::TcpStream;
use tower::Service;
use tracing;

pub struct TcpProxy {
    target_addr: SocketAddr,
    service_ready: bool,
}

impl TcpProxy {
    pub fn new(target_addr: SocketAddr) -> Self {
        TcpProxy {
            target_addr,
            service_ready: true,
        }
    }
}

// TODO: wrap the tcp service with other fall back services that cascade if the main service is not
// ready:
// ```
// impl Service for TcpProxy
// ```
impl Service<TcpStream> for TcpProxy {
    type Response = ();

    type Error = ProxyError;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        if self.service_ready {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }

    fn call(&mut self, mut in_stream: TcpStream) -> Self::Future {
        let target_addr = self.target_addr.clone();

        self.service_ready = false;

        let fut = Box::pin(async move {
            let server_socket = TcpSocket::new_v4().map_err(|e| ProxyError::Outbound(e))?;
            tracing::info!("Attempting to connect to {:?}", &target_addr);
            // Attempt to connect to socket
            let mut out_stream = server_socket
                .connect(target_addr)
                .await
                .map_err(|e| ProxyError::Outbound(e))?;

            let (mut read_in, mut write_in) = in_stream.split();
            let (mut read_out, mut write_out) = out_stream.split();

            // TODO: remove call to tokio::io::copy to a manual call to read from socket and write
            // to socket, because if it fails, it is useful to know if it was the client or the
            // server who ended the connection.
            let client_to_server = async {
                tokio::io::copy(&mut read_in, &mut write_out).await?;
                write_out.shutdown().await?;
                Ok::<(), io::Error>(())
            };

            let server_to_client = async {
                tokio::io::copy(&mut read_out, &mut write_in).await?;
                write_in.shutdown().await?;
                Ok::<(), io::Error>(())
            };

            let (a, b) = tokio::join!(client_to_server, server_to_client);

            // Lets pretend the io calls failed because of the outbound socket
            a.map_err(|e| ProxyError::Outbound(e))?;
            b.map_err(|e| ProxyError::Outbound(e))?;

            Ok(())
        });

        self.service_ready = true;

        fut
    }
}

#[derive(Debug)]
pub enum ProxyError {
    // Errored while reading/writing to inbound socket
    Inbound(io::Error),
    // Errored while reading/writing to outbound socket
    Outbound(io::Error),
}

impl std::fmt::Display for ProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyError::Inbound(inner) => write!(f, "Inbound socket closed: {}", inner),
            ProxyError::Outbound(inner) => write!(f, "Outbound socket closed: {}", inner),
        }
    }
}

impl std::error::Error for ProxyError {}
