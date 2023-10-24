use std::net::{UdpSocket, IpAddr};

use crate::o_node::NodeCreationError;

use super::{Node, config::{Configuration, NodeFunction}};

#[derive(Debug)]
pub struct StdNode {
    socket: UdpSocket,
    port: u16,
    neighbours: Vec<IpAddr>,
}

impl Node for StdNode {
    fn from_configuration(configuration: Configuration) -> Result<Self, NodeCreationError>
    where
        Self: Sized,
    {
        if let NodeFunction::NonBootstraper { bootstraper_ip } = configuration.node_function {
            let socket = UdpSocket::bind(("127.0.0.1", configuration.port))
                .map_err(|err| NodeCreationError::ErrorBindingSocket(err))?;
            println!(
                "Standard Node listening at port {}",
                configuration.port
            );

            socket
                .send_to(b"Neighboors", bootstraper_ip)
                .map_err(|err| NodeCreationError::ErrorBindingSocket(err))?;

            let mut buffer = [0; 1024];
            socket
                .recv(&mut buffer)
                .map_err(|err| NodeCreationError::ErrorConnectingBootstraper(err))?;

            let neighbours = bincode::deserialize(&buffer)
                .map_err(|err| NodeCreationError::ErrorDeserializingIpAddresses(err))?;

            Ok(StdNode {
                socket: socket,
                port: configuration.port,
                neighbours,
            })
        } else {
            panic!("Expected a non bootstraper node configuration");
        }
    }

    fn run(&self) {
        loop {}
    }
}