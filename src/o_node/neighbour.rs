use std::net::IpAddr;

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Neighbour {
    host: IpAddr,
    port: Option<u16>,
}

impl Neighbour {
    pub fn new(ip_address: IpAddr ) -> Self {
        Self {
            host: ip_address,
            port: None,
        }
    }

    pub fn new_with_port(ip_address: IpAddr, port: u16) -> Self {
        Self {
            host: ip_address,
            port: Some(port),
        }
    }

    pub fn address(&self) -> (IpAddr, u16) {
        (self.host, self.port.unwrap_or(8000))
    }
}
