use std::{collections::HashSet, fs::File, io::BufReader};

pub mod server_worker;

pub struct Server {
    port: u16,
}

impl Server {
    pub fn new(port: u16, database_file: &str) -> Self {
        let file = File::open(&database_file).unwrap();

        Self { port }
    }

    pub fn run(&self) {
        let listener = std::net::TcpListener::bind(("127.0.0.1", self.port)).unwrap();

        for stream in listener.incoming() {
            let stream = stream.unwrap();
        }
    }
}
