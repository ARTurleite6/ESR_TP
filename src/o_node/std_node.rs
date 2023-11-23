use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{IpAddr, SocketAddr, TcpStream, UdpSocket},
    sync::Mutex,
    time::Instant,
};

use crate::{
    message::{answer::Answer, query::Query, query::QueryType, Message, Status},
    o_node::{errors::VideoQueryError, NodeCreationError},
};

use super::{
    config::{Configuration, NodeFunction},
    neighbour::Neighbour,
    Node,
};

type ClientAddress = (IpAddr, u16);

#[derive(Debug)]
struct TransmissionChannel {
    clients: Mutex<Vec<ClientAddress>>,
}

impl TransmissionChannel {
    fn new(clients: Vec<ClientAddress>) -> Self {
        Self {
            clients: Mutex::new(clients),
        }
    }

    fn add_client(&self, client: ClientAddress) {
        let mut lock = self.clients.lock().unwrap();
        lock.push(client);
    }

    fn remove_client(&self, client: ClientAddress) {
        let mut lock = self.clients.lock().unwrap();
        lock.retain(|&c| client != c);
    }
}

#[derive(Debug, Default)]
pub struct StdNode {
    port: u16,
    neighbours: Vec<Neighbour>,
    streaming_workers: Mutex<HashMap<String, TransmissionChannel>>,
}

impl StdNode {
    pub fn new(port: u16, neighbours: &[Neighbour]) -> Self {
        Self {
            port,
            neighbours: neighbours.to_owned(),
            ..Default::default()
        }
    }

    fn ask_neighbours(bootstraper_ip: String) -> Result<Answer<Vec<Neighbour>>, NodeCreationError> {
        let query = Query::new(QueryType::Neighbours, None);

        let mut stream = TcpStream::connect(bootstraper_ip)
            .map_err(|error| NodeCreationError::ErrorConnectingBootstraper(error))?;

        stream
            .write(&bincode::serialize(&query).unwrap())
            .map_err(|err| NodeCreationError::ErrorConnectingBootstraper(err))?;

        let mut buffer = [0; 1024];

        stream.read(&mut buffer).unwrap();

        let answer: Answer<Vec<Neighbour>> = bincode::deserialize(&buffer)
            .map_err(|err| NodeCreationError::ErrorDeserializingIpAddresses(err))?;

        return Ok(answer);
    }

    fn find_best_path(
        &self,
        socket: &UdpSocket,
        message: &mut Query,
        addr: SocketAddr,
    ) -> Result<Answer<Vec<u16>>, VideoQueryError> {
        let neighbours: Vec<Neighbour> = self
            .neighbours
            .iter()
            .filter(|neighbour| {
                message
                    .query_type()
                    .file_query()
                    .unwrap()
                    .visited_neighbour(neighbour)
            })
            .map(|neigh| neigh.clone())
            .collect();

        let data = message.query_type_mut().file_query_mut().unwrap();

        data.add_neighbours(&neighbours);

        let message_clone = message.clone();

        let message_encode = bincode::serialize(&message_clone)
            .map_err(|_| VideoQueryError::ErrorDeserializingAnswer)?;

        drop(message_clone);

        for neighbour in &neighbours {
            let neighbour_addr = neighbour.address();
            socket.send_to(&message_encode, neighbour_addr).unwrap();
        }

        let mut answers = Vec::with_capacity(neighbours.len());
        let mut buffer = [0; 1024];
        let begin = Instant::now();
        while answers.len() < neighbours.len() && begin.elapsed().as_secs() < 2 {
            let n = socket.recv(&mut buffer).unwrap();

            let message: Answer<Vec<u16>> = bincode::deserialize(&buffer[..n]).unwrap();

            if message.status().is_ok() {
                answers.push(message);
            }
        }

        return Ok(answers.into_iter().next().unwrap());
    }

    fn handle_video_request(
        &self,
        socket: &UdpSocket,
        mut message: Query,
        addr: SocketAddr,
    ) -> Result<(), VideoQueryError> {
        let lock_guard = self.streaming_workers.lock().unwrap();

        let transmits_file =
            lock_guard.contains_key(message.query_type().file_query().unwrap().file());

        if transmits_file {
            let answer = Answer::from_message(message, vec![self.port], Status::Ok);

            let answer = bincode::serialize(&answer)
                .map_err(|_| VideoQueryError::ErrorDeserializingQuery)?;

            socket.send_to(&answer, addr).unwrap();
        } else {
            let mut selected_answer = self.find_best_path(socket, &mut message, addr).unwrap();

            selected_answer.payload_mut().push(self.port);
            let answer = bincode::serialize(&selected_answer).unwrap();

            socket.send_to(&answer, addr).unwrap();
        }

        todo!();
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
        let _ = std::thread::scope::<_, Result<(), VideoQueryError>>(|s| {
            println!("Standard Node listening at port {}", self.port);
            loop {
                let mut buffer = [0; 1024];

                let result = socket.recv_from(&mut buffer);

                if let Ok((size, addr)) = result {
                    let message: Query = bincode::deserialize(&buffer[..size])
                        .map_err(|_| VideoQueryError::ErrorDeserializingQuery)?;
                    let socket_ref = &socket;
                    s.spawn(
                        move || match self.handle_video_request(socket_ref, message, addr) {
                            Ok(_) => println!("Message handled sucessfully"),
                            Err(error) => eprintln!("Error handling the message {}", error),
                        },
                    );
                } else {
                    eprintln!("Error receing message, no bytes provided");
                }
            }
        });

        return Ok(());
    }

    fn neighbours(&self) -> &[Neighbour] {
        return &self.neighbours;
    }
}
