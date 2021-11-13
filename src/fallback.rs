use std::sync::Arc;
use tower::Service;

// Ideally, this service has other two underlying services. It can recover and fall back (that is,
// it can try to resend its data to another target) only if data __read__ from the original target has
// not been written __to__ the origin. It has an internal buffer where the data written to the
// original target is stored so that it can be retrieved for sending it again.
struct TcpProxyFallback {}
