use crate::error::FlyError;
use crate::proxy::TcpProxy;
use futures::pin_mut;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tower::balance::p2c::Balance;
use tower::discover::ServiceList;
use tower::load::{CompleteOnResponse, PendingRequests};
use tower::util::{CallAllUnordered, ServiceExt};

/// Creates a task that receives a TcpStream and routes it to a series of targets
pub struct DispatchService {
    services: Vec<TcpProxy>,
    rx: mpsc::Receiver<TcpStream>,
}

impl DispatchService {
    pub fn new(rx: mpsc::Receiver<TcpStream>) -> Self {
        DispatchService {
            services: Default::default(),
            rx,
        }
    }

    pub fn with_targets<I>(mut self, targets: I) -> Self
    where
        I: IntoIterator<Item = SocketAddr>,
    {
        for t in targets {
            let service = TcpProxy::new(t);
            self.add_service(service);
        }

        self
    }

    pub fn add_service(&mut self, svc: TcpProxy) {
        self.services.push(svc);
    }

    /// Panics if length of service list == 0
    pub fn build(self, runtime: &Runtime) -> JoinHandle<Result<(), FlyError<TcpStream>>> {
        tracing::debug!("Building dispatch service");
        assert!(self.services.len() != 0, "Services list can't be 0");
        runtime.spawn(async move {
            // (async move {
            tracing::debug!("Dispatch service started");
            let stream_rx = ReceiverStream::new(self.rx);
            let services_with_load =
                self.services
                    .into_iter()
                    .map(|s| -> PendingRequests<TcpProxy> {
                        PendingRequests::new(s, CompleteOnResponse::default())
                    });

            let service_list = ServiceList::new(services_with_load);
            let load_balancer = Balance::new(service_list);

            // pin_mut!(stream_rx);
            let responses = load_balancer.call_all(stream_rx).unordered();
            // let pin = Box::pin(responses);
            while let Some(resp) = responses.next().await {}
            // tracing::debug!("Dispatch service ended");
            Ok(())

            // }
        })
    }
}
