use crate::error::SlyError;
use crate::proxy::TcpProxy;
use crate::target::Target;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tower::balance::p2c::Balance;
use tower::discover::ServiceList;
use tower::load::{CompleteOnResponse, PendingRequests};
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
        I: IntoIterator<Item = Target>,
    {
        for t in targets {
            let service = TcpProxy::new(t.sock_addr());
            self.add_service(service);
        }

        self
    }

    pub fn add_service(&mut self, svc: TcpProxy) {
        self.services.push(svc);
    }

    /// Panics if length of service list == 0
    pub async fn run(self) -> Result<(), SlyError> {
        assert!(self.services.len() != 0, "Services list can't be 0");

        tracing::info!("Dispatch service started");
        let services_with_load = self
            .services
            .into_iter()
            .map(|s| PendingRequests::new(s, CompleteOnResponse::default()));

        let service_list = ServiceList::new(services_with_load);
        let balance = Balance::new(service_list);
        tracing::trace!("Load balancer initialized");

        let stream_rx = ReceiverStream::new(self.rx);
        let mut responses = balance.call_all(stream_rx).unordered();

        while let Some(_) = responses.next().await {
            tracing::trace!("A connection finished");
        }
        tracing::info!("Dispatch service finished");
        Ok(())
    }
}
