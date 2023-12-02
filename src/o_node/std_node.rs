use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{IpAddr, SocketAddr, TcpStream, UdpSocket},
    sync::Mutex,
    time::Instant,
};

use crate::{
    message::{answer::Answer, query::Query, query::{QueryType, self}, Message, Status},
    o_node::{errors::VideoQueryError, NodeCreationError}, server::server_worker::streaming_intermediate_worker::{StreamingWorker, TransmissionChannel},
};

use super::{
    config::{Configuration, NodeFunction},
    neighbour::Neighbour,
    Node,
};

type ClientAddress = (IpAddr, u16);

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
        message: &mut Query,
    ) -> Result<(Answer<Vec<Neighbour>>, SocketAddr), VideoQueryError> {
        let neighbours: Vec<Neighbour> = self
            .neighbours
            .iter()
            .filter(|neighbour| {
                !message
                    .query_type()
                    .file_query()
                    .unwrap()
                    .visited_neighbour(neighbour)
            })
            .map(|neigh| neigh.clone())
            .collect();

        dbg!(&neighbours);

        let data = message.query_type_mut().file_query_mut().unwrap();

        data.add_neighbours(&neighbours);

        let message_clone = message.clone();

        let message_encode = bincode::serialize(&message_clone)
            .map_err(|_| VideoQueryError::ErrorDeserializingAnswer)?;

        drop(message_clone);
        
        let query_socket = UdpSocket::bind(("0.0.0.0", 0)).unwrap();

        for neighbour in &neighbours {
            let neighbour_addr = neighbour.address();
            query_socket.send_to(&message_encode, neighbour_addr).unwrap();
        }

        let mut answers = Vec::with_capacity(neighbours.len());
        let mut buffer = [0; 1024];
        let begin = Instant::now();
        while answers.len() < neighbours.len() && begin.elapsed().as_secs() < 2 {
            let (n, addr) = query_socket.recv_from(&mut buffer).unwrap();

            let message: Answer<Vec<Neighbour>> = bincode::deserialize(&buffer[..n]).unwrap();

            if message.status().is_ok() {
                answers.push((message, addr));
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

        let answer = if transmits_file {
            let answer = Answer::<Vec<Neighbour>>::from_message(message, Vec::new(), Status::Ok);

            bincode::serialize(&answer).map_err(|_| VideoQueryError::ErrorDeserializingQuery)?
        } else {
            let (mut selected_answer, server_addr) = self.find_best_path(&mut message).unwrap();
            dbg!(&selected_answer);

            selected_answer
                .payload_mut()
                .push(Neighbour::from(server_addr));

            bincode::serialize(&selected_answer)
                .map_err(|_| VideoQueryError::ErrorDeserializingQuery)?
        };

        socket.send_to(&answer, addr).unwrap();

        return Ok(());#[derive(Debug)]
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
        dbg!(self);
        let socket = UdpSocket::bind(("0.0.0.0", self.port))
            .map_err(|err| NodeCreationError::ErrorBindingSocket(err))?;

        let _ = std::thread::scope(|s| {
            println!("Standard Node listening at port {}", self.port);
            s.spawn(|| loop {
                let mut buffer = [0; 1024];

                let result = socket.recv_from(&mut buffer);
                dbg!(&result);

                if let Ok((size, addr)) = result {
                    let message: Query = bincode::deserialize(&buffer[..size])
                        .map_err(|_| VideoQueryError::ErrorDeserializingQuery).expect("Error deserializing message");
                    dbg!(&message);
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
            });
            
            s.spawn(|| {
                StreamingWorker::new(self.port, &self.streaming_workers)
                    .run();
            });
        });

        return Ok(());
    }

    fn neighbours(&self) -> &[Neighbour] {
        return &self.neighbours;
    }
}
