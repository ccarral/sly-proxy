# sly-proxy

Configurable, tokio/tower-built tcp proxy that can __listen__ on a configurable number
of ports and forward plain tcp connections to a series of configurable targets.
If the proxy couldn't reach its original target, it will seamlessly fall back
to another target.

# TODO

* Add dns resolving.
* Buffer data written to target so that if the target unexpectedly closes (and
  no response data has been written back to client) we can resend the data to 
  another target with the client being none the wiser about the broken connection.
* Instead of only one fallback target, wrap the other targets with an
  Arc<Mutex<>> so that the proxy service has the whole range of 
  the other available targets to choose from.
* Dynamic discovery service that implements `tower::discover::Discover`. 
