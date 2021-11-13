use serde::Deserialize;
use std::net::{IpAddr, SocketAddr};

#[derive(Deserialize)]
pub struct Target {
    ip: IpAddr,
    port: u16,
}

impl Target {
    pub fn sock_addr(&self) -> SocketAddr {
        SocketAddr::new(self.ip, self.port)
    }

    pub fn from_sock_addr(address: &SocketAddr) -> Self {
        Target {
            ip: address.ip(),
            port: address.port(),
        }
    }
}
