use crate::error::FlyError;
use crate::proxy::TcpProxy;
use futures::pin_mut;
use futures::{Future, FutureExt};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tower::balance::p2c::Balance;
use tower::builder::ServiceBuilder;
use tower::discover::ServiceList;
use tower::limit::ConcurrencyLimit;
use tower::load::{CompleteOnResponse, Constant, PendingRequests};
use tower::util::CallAll;
use tower::Service;
use tower::ServiceExt;

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
    pub async fn run(mut self) -> Result<(), FlyError<TcpStream>> {
        assert!(self.services.len() != 0, "Services list can't be 0");
        // async move {
        tracing::info!("Dispatch service started");
        // tracing::info!("Streamed rx initialized");
        // let services_with_load = self.services.into_iter().map(|s| Constant::new(s, 0));

        // let service_list = ServiceList::new(services_with_load);

        // let balance = Balance::new(service_list);
        // let load_balancer = ConcurrencyLimit::new(load_balancer, 1);
        tracing::info!("Load balancer initialized");

        // let stream_rx = ReceiverStream::new(self.rx);
        // let mut responses = balance.call_all(stream_rx).unordered();
        // if let Some(_s) = responses
        // .next()
        // .inspect(|f| tracing::info!("Polling responses: {:?}", f))
        // .await
        // {
        // tracing::info!("Packet forwarded");
        // }
        while let Some(tcp) = self.rx.recv().await {
            tracing::info!("Received tcp stream");
            let mut svc = TcpProxy::new("127.0.0.1:8080".parse().unwrap());
            svc.ready().await?.call(tcp).await?;
        }
        // tracing::info!("Got here");

        // Select from unoccupied services

        // while let Some(_) = responses.next().await {
        // tracing::debug!("A connection finished");
        // }
        tracing::info!("Dispatch service finished");

        Ok(())
    }
}
