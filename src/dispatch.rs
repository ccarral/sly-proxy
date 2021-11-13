use std::io;
use std::pin::Pin;

use crate::error::SlyError;
use crate::proxy::TcpProxy;
use crate::target::Target;
use futures::Future;
use futures_util::future;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tower::balance::p2c::Balance;
use tower::discover::ServiceList;
use tower::load::{CompleteOnResponse, PendingRequests};
use tower::retry::{Policy, Retry};
use tower::{Service, ServiceExt};

/// Creates a task that receives a TcpStream and routes it to a series of targets
pub struct DispatchService {
    services: Vec<TcpProxy>,
    rx: mpsc::Receiver<TcpStream>,
}

struct DispatchPolicy {
    attempts: usize,
    limit: usize,
}

impl DispatchPolicy {
    fn with_attempts_limit(limit: usize) -> Self {
        DispatchPolicy { attempts: 0, limit }
    }
}

type Req = TcpStream;
type Res = ();
type Error = io::Error;

// Checks if the service has at least been retried n number of times where n = the number of
// underlying services available
impl Policy<Req, Res, Error> for DispatchPolicy {
    type Future = Pin<Box<dyn Future<Output = Self>>>;

    fn retry(&self, _req: &Req, _result: Result<&Res, &Error>) -> Option<Self::Future> {
        // What errors can we encounter when dealing with sockets?
        // * Unexpected disconnection
        // * ...?
        // I currently can't think any that dont merit adding to the count

        if self.attempts >= self.limit {
            None
        } else {
            // This is... quite the return value
            Some(Box::pin(future::ready(DispatchPolicy {
                attempts: self.attempts + 1,
                limit: self.limit,
            })))
        }
    }

    fn clone_request(&self, _req: &Req) -> Option<Req> {
        None
    }
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

        let stream_rx = ReceiverStream::new(self.rx);
        let mut responses = balance.call_all(stream_rx).unordered();

        while let Some(_) = responses.next().await {
            tracing::trace!("a connection finished");
        }
        tracing::info!("dispatch service finished");
        Ok(())
    }
}
