use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read, Write},
    net::{IpAddr, TcpListener, TcpStream},
    str::FromStr,
};

use crate::message::{answer::Answer, query::Query, Status};

use super::{
    config::{Configuration, NodeFunction},
    neighbour::Neighbour,
    std_node::StdNode,
    Node, NodeCreationError,
};

#[derive(Debug, Default)]
pub struct BootstraperNode {
    bootstraping_port: u16,
    topology: HashMap<IpAddr, Vec<Neighbour>>,
    std_node: StdNode,
}

impl BootstraperNode {
    fn boostraping_service(&self, mut stream: TcpStream) {
        let mut buffer = [0; 1024];

        stream.read(&mut buffer).unwrap();

        let message: Query = bincode::deserialize(&buffer).expect("Error deserializing message");

        let ip_client = stream.peer_addr().unwrap().ip();

        let neighbours = self.topology.get(&ip_client).unwrap();

        let answer = Answer::from_message(message, neighbours.to_owned(), Status::Ok);

        let _ = stream.write(&bincode::serialize(&answer).unwrap());
    }
}

impl Node for BootstraperNode {
    fn from_configuration(configuration: Configuration) -> Result<Self, NodeCreationError>
    {
        if let NodeFunction::Bootstraper { ref topology, port } = configuration.node_function {
            let file = File::open(topology)
                .map_err(|_err| NodeCreationError::InexistentTopology(topology.clone()))?;

            let topology: HashMap<IpAddr, Vec<Neighbour>> =
                serde_json::from_reader(BufReader::new(file)).unwrap();

            let ip = IpAddr::from_str("127.0.0.1").unwrap();

            let neighbours = topology
                .get(&ip)
                .expect("Error getting my own neighbours")
                .clone();

            let std_node = StdNode::new(configuration.port, &neighbours);

            return Ok(BootstraperNode {
                bootstraping_port: port,
                topology,
                std_node,
            });
        } else {
            panic!("Expected a bootstraper node configuration");
        }
    }

    fn run(&self) -> Result<(), NodeCreationError> {
        std::thread::scope(|s| {
            //bootstraping thread
            s.spawn(|| {
                let socket = TcpListener::bind(("127.0.0.1", self.bootstraping_port))
                    .expect("Error binding bootstraping socket");
                println!(
                    "Bootstraper Node listening at port {}",
                    self.bootstraping_port
                );

                for stream in socket.incoming() {
                    self.boostraping_service(stream.unwrap());
                }
            });

            //std thread
            s.spawn(|| {
                self.std_node.run()
            });
        });

        Ok(())
    }

    fn neighbours(&self) -> &[super::neighbour::Neighbour] {
        return &self.std_node.neighbours();
    }
}
