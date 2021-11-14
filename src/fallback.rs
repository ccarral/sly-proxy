use crate::proxy::{ProxyError, TcpProxy};
use futures::Future;
use std::{io, net::SocketAddr, pin::Pin};
use tokio::net::{TcpSocket, TcpStream};
use tower::{Service, ServiceExt};

// Ideally, this service has other underlying service. It can recover and fall back (that is,
// it can try to resend its data to another target) only if data __read__ from the original target has
// not been written __to__ the origin. It has an internal buffer where the data written to the
// original target is stored so that it can be retrieved for sending it again.
// However, for now we will settle for a service that builds a TcpProxy depending on wether the
// target is available. Else, it will fall back.
//
// Another idea for the fallback service is a graph that connects all of the servers in a  n x n
// fashion and everyone has everyone else with a mutex. Calls to ready() return Pending when the
// mutex is being held by someone else. When a service tries to make a connection and fails, it
// searches for the next unlocked available service and runs with it.
pub struct TcpProxyFallback {
    target_addr: SocketAddr,
    fall_back_addr: Option<SocketAddr>,
}

impl TcpProxyFallback {
    pub fn new(target_addr: SocketAddr) -> Self {
        TcpProxyFallback {
            target_addr,
            fall_back_addr: None,
        }
    }

    pub fn with_fallback(&mut self, fall_back_addr: SocketAddr) -> &mut Self {
        self.fall_back_addr = Some(fall_back_addr);
        self
    }
}

impl Service<TcpStream> for TcpProxyFallback {
    type Response = ();

    type Error = ProxyError;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, req: TcpStream) -> Self::Future {
        // First, try to connect to target address
        async fn connect_to_address(addr: SocketAddr) -> Result<TcpStream, io::Error> {
            tracing::info!("trying to connect to {}", addr);
            let target = match addr {
                SocketAddr::V4(_) => {
                    let sock = TcpSocket::new_v4().unwrap();
                    sock.connect(addr).await?
                }
                SocketAddr::V6(_) => {
                    let sock = TcpSocket::new_v6().unwrap();
                    sock.connect(addr).await?
                }
            };
            Ok(target)
        }

        let fallback_address = self.fall_back_addr;
        let target_address = self.target_addr;

        Box::pin(async move {
            let actual_target = match connect_to_address(target_address).await {
                Ok(t) => Ok(t),
                Err(e) => {
                    match fallback_address {
                        Some(fall_back) => {
                            tracing::info!(
                                "failed to connect to {}, retryng with {}",
                                target_address,
                                fall_back
                            );
                            // Main target failed for whatever reason, fall back
                            match connect_to_address(fall_back).await {
                                Ok(t) => Ok(t),
                                // Fall back failed, there is no god
                                Err(e) => Err(e),
                            }
                        }
                        None => Err(e),
                    }
                }
            }
            .map_err(|e| ProxyError::Outbound(e))?;

            let mut proxy = TcpProxy::new();

            proxy.ready().await?.call((req, actual_target)).await?;

            Ok(())
        })
    }
}
