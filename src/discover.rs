use crate::proxy::TcpProxy;
use futures::Stream;
use pin_project_lite::pin_project;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::Poll;
use std::time::{Duration, Instant};
use tokio::time::Interval;
use tower::discover::Change;

pin_project! {
    struct TargetDiscover<O, K, V> {
        last_checked: Instant,
        lapse: Duration,
        #[pin]
        orchestration_service: O,
        _phantom_key: PhantomData<K>,
        _phantom_v: PhantomData<V>,
    }
}

struct OrchestrationRequest(Instant);

// We will borrow the Change enum
struct OrchestrationResponse<K, V>(Option<Change<K, V>>);

/// Trait for a service that receives a request and returns a list of available services.
trait OrchestrationService<K: Eq, V> {
    // Check inner storage and return new (or removed)
    // services from that point onwards.
    // This is where we might make an rpc service call
    fn poll_services(
        self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        _req: OrchestrationRequest,
    ) -> Poll<Result<OrchestrationResponse<K, V>, OrchestrationServiceError>> {
        todo!();
    }
}

// If we implement Stream, the Discover trait will take care of the rest
impl<O, K, V> Stream for TargetDiscover<O, K, V>
where
    K: Eq,
    O: OrchestrationService<K, V>,
{
    type Item = Result<Option<TcpProxy>, OrchestrationServiceError>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.project();
        let inner_service = this.orchestration_service;
        // Create timer

        let lapse = this.lapse;

        let mut interval: Interval = tokio::time::interval(*lapse);

        match interval.poll_tick(cx) {
            Poll::Ready(_) => {
                // Is ready to check on services
                let req = OrchestrationRequest(Instant::now());
                match inner_service.poll_services(cx, req) {
                    Poll::Ready(resp) => {
                        let OrchestrationResponse(change) = resp?;
                        match change {
                            Some(_c) => {
                                // Return new services
                                // Construct a new target
                                // Create Service From target
                                // Yield target with Poll::Ready(Ok(target))
                                todo!();
                            }
                            None => Poll::Pending,
                        }
                    }
                    Poll::Pending => {
                        return Poll::Pending;
                    }
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

#[derive(Debug)]
struct OrchestrationServiceError {}

impl std::error::Error for OrchestrationServiceError {}

impl std::fmt::Display for OrchestrationServiceError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unimplemented!();
    }
}
