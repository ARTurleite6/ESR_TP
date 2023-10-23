#![allow(dead_code)]

pub mod config;
use std::{collections::HashMap, fs::File, io::Read, net::{UdpSocket, IpAddr}};

use config::{Configuration, NodeFunction};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NodeCreationError {
    #[error("The file passed to create topology does not exist, topology: {0}")]
    InexistentTopology(String),
    #[error("Error binding socket: {0}")]
    ErrorBindingSocket(std::io::Error),
    #[error("Error connecting to bootstraper: {0}")]
    ErrorConnectingBootstraper(std::io::Error),
    #[error("Error deserializing the ip addresses")]
    ErrorDeserializingIpAddresses(bincode::Error),
}

#[derive(Debug)]
pub enum Node {
    Std(StdNode),
    Bootstraper(BootstraperNode),
}

impl Node {
    pub fn from_configuration(configuration: Configuration) -> Result<Self, NodeCreationError> {

            match configuration.node_function {
            NodeFunction::Bootstraper { ref topology } => {
                let mut file = File::open(topology)
                    .map_err(|_err| NodeCreationError::InexistentTopology(topology.clone()))?;

                let mut buffer = String::default();

                file.read_to_string(&mut buffer)
                    .map_err(|err| NodeCreationError::InexistentTopology(err.to_string()))?;

                let topology = buffer.lines().map(|line| {
                    let mut camps = line.split(":");
                    let key = camps.next().expect("Expected key for node IP");

                    let values = camps.next().expect("Expected ip addresses that are the neighboors of given node");

                    let ips = values.split(",");

                    let neighboors = ips.map(|value| value.trim().to_string().parse().expect("Error parsing Ip Address")).collect();
                        
                    (key.parse().expect("Error parsing ip address"), neighboors)
                }).collect();

                let socket = UdpSocket::bind(("127.0.0.1", configuration.port)).map_err(|err| NodeCreationError::ErrorBindingSocket(err))?;
                println!("Bootstrapper Node listening at port {}", configuration.port);
                Ok(Self::Bootstraper(BootstraperNode { socket, port: configuration.port, topology }))

            }
            NodeFunction::NonBootstraper { bootstraper_ip } => {
                let socket = UdpSocket::bind(("127.0.0.1", configuration.port)).map_err(|err| NodeCreationError::ErrorBindingSocket(err))?;
                println!("Non Bootstrapper Node listening at port {}", configuration.port);

                socket.send_to(b"Neighboors", bootstraper_ip).map_err(|err| NodeCreationError::ErrorBindingSocket(err))?;

                let mut buffer = [0; 1024];
                socket.recv(&mut buffer).map_err(|err| NodeCreationError::ErrorConnectingBootstraper(err))?;

                let neighbours = bincode::deserialize(&buffer).map_err(|err| NodeCreationError::ErrorDeserializingIpAddresses(err))?;

                Ok(Self::Std(StdNode { socket: socket, port: configuration.port, neighbours }))
            },
        }

    }

    pub fn run(&self) {
        loop {
            
        } 
    }
}

#[derive(Debug)]
pub struct StdNode {
    socket: UdpSocket,
    port: u16,
    neighbours: Vec<IpAddr>,
}

#[derive(Debug)]
pub struct BootstraperNode {
    socket: UdpSocket,
    port: u16,
    topology: HashMap<IpAddr, Vec<IpAddr>>,
}