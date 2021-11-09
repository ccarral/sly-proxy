use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::Poll;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpSocket;
use tokio::net::TcpStream;
use tracing;
//
use tower::Service;

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

impl Service<TcpStream> for TcpProxy {
    type Response = ();

    type Error = io::Error;

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
            let server_socket = TcpSocket::new_v4().expect("Couldn't initialize socket");
            tracing::info!("Attempting to connect to {:?}", &target_addr);
            // Attempt to connect to socket
            let mut out_stream = server_socket.connect(target_addr).await.unwrap();

            let (mut read_in, mut write_in) = in_stream.split();
            let (mut read_out, mut write_out) = out_stream.split();

            let client_to_server = async {
                tokio::io::copy(&mut read_in, &mut write_out).await.unwrap();
                write_out.shutdown().await.unwrap();
            };

            let server_to_client = async {
                tokio::io::copy(&mut read_out, &mut write_in).await.unwrap();
                write_in.shutdown().await.unwrap();
            };

            tokio::join!(client_to_server, server_to_client);

            Ok(())
        });

        self.service_ready = true;

        fut
    }
}
