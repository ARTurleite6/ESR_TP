use std::net::{UdpSocket, IpAddr};

use crate::o_node::{NodeCreationError, message::{query::{Query, QueryType}, answer::Answer, Message}};

use super::{Node, config::{Configuration, NodeFunction}};

#[derive(Debug)]
pub struct StdNode {
    socket: UdpSocket,
    port: u16,
    neighbours: Vec<IpAddr>,
}

impl StdNode {
    pub fn new(socket: UdpSocket, port: u16, neighbours: &[IpAddr]) -> Self {
        Self {
            socket,
            port,
            neighbours: neighbours.to_owned(),
        }
    }
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

            let query = Query::new(QueryType::Neighbours, None);
            dbg!(&query);

            socket
                .send_to(&bincode::serialize(&query).unwrap(), bootstraper_ip)
                .map_err(|err| NodeCreationError::ErrorBindingSocket(err))?;

            let mut buffer = [0; 1024];
            socket
                .recv(&mut buffer)
                .map_err(|err| NodeCreationError::ErrorConnectingBootstraper(err))?;

            let answer: Answer<Vec<IpAddr>> = bincode::deserialize(&buffer)
                .map_err(|err| NodeCreationError::ErrorDeserializingIpAddresses(err))?;

            let neighbours = answer.payload().expect("Expected a payload");

            Ok(StdNode::new(
                socket,
                configuration.port,
                neighbours,
            ))
        } else {
            panic!("Expected a non bootstraper node configuration");
        }
    }

    fn run(&self) {
        loop {}
    }
}