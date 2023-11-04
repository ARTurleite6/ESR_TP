use std::{
    io::{Read, Write},
    net::{IpAddr, TcpStream, UdpSocket},
};

use crate::o_node::{
    message::{
        answer::Answer,
        query::{Query, QueryType},
        Message,
    },
    NodeCreationError,
};

use super::{
    config::{Configuration, NodeFunction},
    Node,
};

#[derive(Debug)]
pub struct StdNode {
    port: u16,
    neighbours: Vec<IpAddr>,
}

impl StdNode {
    pub fn new(port: u16, neighbours: &[IpAddr]) -> Self {
        Self {
            port,
            neighbours: neighbours.to_owned(),
        }
    }

    fn ask_neighbours(bootstraper_ip: String) -> Result<Answer<Vec<IpAddr>>, NodeCreationError> {
        let query = Query::new(QueryType::Neighbours, None);
        dbg!(&query);

        let mut stream = TcpStream::connect(bootstraper_ip)
            .map_err(|error| NodeCreationError::ErrorConnectingBootstraper(error))?;

        dbg!(&stream);

        stream
            .write(&bincode::serialize(&query).unwrap())
            .map_err(|err| NodeCreationError::ErrorConnectingBootstraper(err))?;

        let mut buffer = [0; 1024];

        stream.read(&mut buffer).unwrap();

        let answer: Answer<Vec<IpAddr>> = bincode::deserialize(&buffer)
            .map_err(|err| NodeCreationError::ErrorDeserializingIpAddresses(err))?;

        return Ok(answer);
    }
}

impl Node for StdNode {
    fn from_configuration(configuration: Configuration) -> Result<Self, NodeCreationError>
    where
        Self: Sized,
    {
        if let NodeFunction::NonBootstraper { bootstraper_ip } = configuration.node_function {
            let answer = StdNode::ask_neighbours(bootstraper_ip)?;

            dbg!(&answer);

            let neighbours = answer.payload().expect("Expected payload");

            Ok(StdNode::new(configuration.port, neighbours))
        } else {
            panic!("Expected a non bootstraper node configuration");
        }
    }

    fn run(&self) -> Result<(), NodeCreationError> {
        let socket = UdpSocket::bind(("127.0.0.1", self.port))
            .map_err(|err| NodeCreationError::ErrorBindingSocket(err))?;
        println!("Standard Node listening at port {}", self.port);
        loop {}
    }
}
