use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Deserialize)]
pub struct Target(pub SocketAddr);
