use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Neighbour {
    host: IpAddr,
    port: u16,
}

impl From<SocketAddr> for Neighbour {
    fn from(value: SocketAddr) -> Self {
        Self {
            host: value.ip(),
            port: value.port(),
        }
    }
}

impl FromStr for Neighbour {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();

        if parts.len() > 2 || parts.is_empty() {
            return Err("Invalid input Format. Expected: ip_addr, port".to_string());
        }

        let ip_addr = IpAddr::from_str(parts[0].trim()).map_err(|err| err.to_string())?;

        let port = parts
            .get(1)
            .unwrap_or(&"8000")
            .parse::<u16>()
            .map_err(|e| e.to_string())?;

        Ok(Neighbour {
            host: ip_addr,
            port,
        })
    }
}

impl Neighbour {
    pub fn new(ip_address: IpAddr) -> Self {
        Self {
            host: ip_address,
            port: 8000,
        }
    }

    pub fn new_with_port(ip_address: IpAddr, port: u16) -> Self {
        Self {
            host: ip_address,
            port,
        }
    }

    pub fn address(&self) -> (IpAddr, u16) {
        (self.host, self.port)
    }
}
