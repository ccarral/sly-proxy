use crate::error::SlyError;
use crate::fallback::TcpProxyFallback;
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
    services: Vec<TcpProxyFallback>,
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
        let mut targets = targets.into_iter();

        loop {
            if let Some(t) = targets.next() {
                let target_addr = t.sock_addr();
                let mut fallback = TcpProxyFallback::new(target_addr);
                if let Some(f) = targets.next() {
                    let fallback_addr = f.sock_addr();
                    fallback.with_fallback(fallback_addr);
                }
                self.add_service(fallback);
            } else {
                break;
            }
        }

        self
    }

    pub fn add_service(&mut self, svc: TcpProxyFallback) {
        self.services.push(svc);
    }

    /// Panics if length of service list == 0
    pub async fn run(self) -> Result<(), SlyError> {
        assert!(self.services.len() != 0, "Services list can't be 0");

        tracing::info!("dispatch service started");
        let services_with_load = self
            .services
            .into_iter()
            // Marks a service as ready to receive new requests once it has completed its future
            .map(|s| PendingRequests::new(s, CompleteOnResponse::default()));

        let service_list = ServiceList::new(services_with_load);

        let balance = Balance::new(service_list);
        tracing::trace!("load balancer initialized");

        // Wraps the Balance service in a Retry service that takes a failed request (for whatever
        // reason) and retries it
        // let retry = Retry::new(DispatchPolicy::with_attempts_limit(3), balance);

        // let make_proxy = service_fn(|addr: SocketAddr| async {});

        let stream_rx = ReceiverStream::new(self.rx);
        let mut responses = balance.call_all(stream_rx).unordered();

        loop {
            match responses.try_next().await {
                Ok(opt) => match opt {
                    Some(_) => (),
                    None => {
                        // None means there are no more connections to process so... we are done?
                    }
                },
                Err(e) => tracing::error!("{}", e),
            }
        }
    }
}
