use std::{collections::HashSet, fs::File, io::BufReader};

use gtk::gdk::keys::constants::W;

use self::server_worker::ServerWorker;

pub mod server_worker;

pub struct Server {
    port: u16,
}

impl Server {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn run(&self) {
        let listener = std::net::TcpListener::bind(("127.0.0.1", self.port)).unwrap();
        println!("Listening on port {}", self.port);

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            std::thread::spawn(move || {
                let mut worker = ServerWorker::new(stream);
                worker.run();
            });
        }
    }
}
