use std::future::Future;
use std::io;
use std::pin::Pin;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tower::Service;
use tracing;

pub struct TcpProxy {}

impl TcpProxy {
    pub fn new() -> Self {
        TcpProxy {}
    }
}

impl Service<(TcpStream, TcpStream)> for TcpProxy {
    type Response = ();

    type Error = ProxyError;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, (mut in_stream, mut out_stream): (TcpStream, TcpStream)) -> Self::Future {
        let fut = Box::pin(async move {
            let (mut read_in, mut write_in) = in_stream.split();
            let (mut read_out, mut write_out) = out_stream.split();

            // TODO: remove call to tokio::io::copy to a manual call to read from socket and write
            // to socket, because if it fails, it is useful to know if it was the client or the
            // server who ended the connection.
            let client_to_server = async {
                tokio::io::copy(&mut read_in, &mut write_out).await?;
                write_out.shutdown().await?;
                tracing::info!("outbound socket closed");
                Ok::<(), io::Error>(())
            };

            let server_to_client = async {
                tokio::io::copy(&mut read_out, &mut write_in).await?;
                tracing::info!("inbound socket closed");
                write_in.shutdown().await?;
                Ok::<(), io::Error>(())
            };

            let (a, b) = tokio::join!(client_to_server, server_to_client);

            // Lets pretend the io calls failed because of the outbound socket
            a.map_err(|e| ProxyError::Outbound(e))?;
            b.map_err(|e| ProxyError::Outbound(e))?;

            Ok(())
        });

        fut
    }
}

#[derive(Debug)]
pub enum ProxyError {
    // Errored while reading/writing to inbound socket
    #[allow(dead_code)]
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
