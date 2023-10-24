use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    net::{IpAddr, UdpSocket},
};

use crate::o_node::message::{Message, Status};

use super::{
    config::{Configuration, NodeFunction},
    Node, NodeCreationError,
};

#[derive(Debug)]
pub struct BootstraperNode {
    std_port: u16,
    bootstraping_port: u16,
    topology: HashMap<IpAddr, Vec<IpAddr>>,
    neighbours: Vec<IpAddr>,
}

impl Node for BootstraperNode {
    fn from_configuration(configuration: Configuration) -> Result<Self, NodeCreationError>
    where
        Self: Sized,
    {
        if let NodeFunction::Bootstraper { ref topology, port } = configuration.node_function {
            let file = File::open(topology)
                .map_err(|_err| NodeCreationError::InexistentTopology(topology.clone()))?;

            let topology: HashMap<IpAddr, Vec<IpAddr>> =
                serde_json::from_reader(BufReader::new(file)).unwrap();

            let socket = UdpSocket::bind(("127.0.0.1", configuration.port))
                .map_err(|err| NodeCreationError::ErrorBindingSocket(err))?;
            println!("Bootstrapper Node listening at port {}", configuration.port);

            let ip = socket
                .local_addr()
                .expect("Error getting my own ip address")
                .ip();
            dbg!(&ip);
            let neighbours = topology
                .get(&ip)
                .expect("Error getting my own neighbours")
                .clone();
            dbg!(&neighbours);

            return Ok(BootstraperNode {
                std_port: configuration.port,
                bootstraping_port: port,
                topology,
                neighbours,
            });
        } else {
            panic!("Expected a bootstraper node configuration");
        }
    }

    fn run(&self) {
        std::thread::scope(|s| {
            //bootstraping thread
            s.spawn(|| {
                let socket = UdpSocket::bind(("127.0.0.1", self.bootstraping_port))
                    .expect("Error binding bootstraping socket");
                println!(
                    "Bootstraper Node listening at port {}",
                    self.bootstraping_port
                );

                let mut buffer = [0; 1024];

                loop {
                    let (_, addr) = socket
                        .recv_from(&mut buffer)
                        .expect("Error receiving from bootstraping socket");

                    let message: Message =
                        bincode::deserialize(&buffer).expect("Error deserializing message");
                    dbg!(&message);

                    if let Some(neighbours) = self.topology.get(&addr.ip()) {
                        let asnwer = Message::answer(message.id(), "Your neighboors are".to_string(), Status::Ok);
                        socket.send_to(b"Neighbours not found for your address", addr)
                    } else {
                    }
                }
            });
        })
    }
}
