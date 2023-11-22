use std::net::IpAddr;

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Neighbour {
    ip_address: IpAddr,
    port: u16,
}

impl Neighbour {
    pub fn new(ip_address: IpAddr ) -> Self {
        Self {
            ip_address,
            port: 8000,
        }
    }

    pub fn new_with_port(ip_address: IpAddr, port: u16) -> Self {
        Self {
            ip_address,
            port
        }
    }

    pub fn address(&self) -> (IpAddr, u16) {
        (self.ip_address, self.port)
    }
}
