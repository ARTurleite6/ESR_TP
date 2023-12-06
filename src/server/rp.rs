use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpStream, UdpSocket},
    sync::{Arc, Mutex},
    time::Instant,
};

use clap::Parser;

use crate::{
    message::{
        answer::Answer,
        metrics::{MetricsRequest, MetricsResponse},
        query::Query,
        Status,
    },
    o_node::neighbour::Neighbour,
};

use super::{
    server_worker::streaming_intermediate_worker::StreamingWorker,
    transmission_channel::TransmissionChannel,
};

#[derive(Debug, Parser)]
pub struct RPArgs {
    #[clap(short, long, default_value = "8554")]
    port: u16,
    #[clap(short, long)]
    servers: Vec<Neighbour>,
}

#[derive(Debug)]
pub struct RP {
    content_servers: Vec<Neighbour>,
    port: u16,
    transmission_workers: Mutex<HashMap<String, TransmissionChannel>>,
}

impl RP {
    pub fn new(args: RPArgs) -> Self {
        Self {
            content_servers: args.servers,
            port: args.port,
            transmission_workers: Mutex::new(HashMap::new()),
        }
    }

    fn video_query_service(&self, server_connections: Vec<TcpStream>) {
        let udp_socket = Arc::new(UdpSocket::bind(("0.0.0.0", self.port)).unwrap());

        let mut buffer = [0; 1024];
        println!(
            "Video query service listening on port {}",
            udp_socket.local_addr().unwrap().port()
        );

        let server_connections: Arc<Vec<Mutex<TcpStream>>> = Arc::new(
            server_connections
                .into_iter()
                .map(|stream| Mutex::new(stream))
                .collect(),
        );

        std::thread::scope(|s| loop {
            let (n, addr) = udp_socket.recv_from(&mut buffer).unwrap();

            if n == 0 {
                continue;
            }

            let message = &buffer[..n];

            let query: Query = bincode::deserialize(message).unwrap();

            let server_connections = Arc::clone(&server_connections);

            let udp_socket = Arc::clone(&udp_socket);
            s.spawn(move || {
                let video = query
                    .query_file()
                    .expect("Expected file on query")
                    .to_string();

                let workers = self.transmission_workers.lock().unwrap();
                if workers.contains_key(&video) {
                    let answer: Answer<Vec<Neighbour>> =
                        Answer::from_message(query, Vec::new(), Status::Ok);

                    let answer = bincode::serialize(&answer).expect("Error serializing packet");

                    udp_socket.send_to(&answer, addr).unwrap();
                } else {
                    let request = MetricsRequest::new(video.to_string());
                    let request = bincode::serialize(&request).expect("Error serializing packet");

                    for server in server_connections.iter() {
                        dbg!(&server);
                        let mut server = server.lock().unwrap();
                        let n = server.write(&request).unwrap();
                        dbg!(n);
                    }

                    let now = Instant::now();
                    let mut answers = Vec::new();
                    let mut count = 0;
                    dbg!(server_connections.len());
                    while count < server_connections.len() && now.elapsed().as_secs() < 5 {
                        let mut buffer = [0; 1024];
                        let mut server = server_connections[count].lock().unwrap();
                        let n = server.read(&mut buffer).unwrap();
                        if n == 0 {
                            continue;
                        }
                        let response: MetricsResponse =
                            bincode::deserialize(&buffer).expect("Error deserializing packet");
                        dbg!(&response);

                        let neighbour = Neighbour::new_with_port(
                            server.peer_addr().unwrap().ip(),
                            response.streaming_port(),
                        );

                        answers.push((response, neighbour));

                        count += 1;
                    }
                    println!("Answers: {:?}", &answers);

                    dbg!(&answers);
                    let server_to_use = answers.into_iter().nth(0);

                    let answer = if let Some(server) = server_to_use {
                        let server_to_use = server.1;
                        Answer::from_message(query, vec![server_to_use], Status::Ok)
                    } else {
                        Answer::from_message(query, vec![], Status::VideoNotFound)
                    };

                    let answer = bincode::serialize(&answer).expect("Error serializing packet");

                    dbg!((&answer, &addr));

                    let n = udp_socket.send_to(&answer, addr).unwrap();
                    dbg!(n);
                }
            });
        });
    }

    pub fn run(&self) {
        let server_connections: Vec<TcpStream> = self
            .content_servers
            .iter()
            .map(|server| {
                println!("Connecting to server: {:?}", server);
                TcpStream::connect(server.address()).unwrap()
            })
            .collect();

        std::thread::scope(|s| {
            s.spawn(|| {
                self.video_query_service(server_connections);
            });

            s.spawn(|| {
                StreamingWorker::new(self.port, &self.transmission_workers).run();
            });
        });
    }
}
