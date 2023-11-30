use std::{
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Mutex},
};

#[derive(Debug)]
pub struct TransmissionChannelWorker {
    server_addr: SocketAddr,
    socket: Arc<UdpSocket>,
    addresses: Mutex<Vec<SocketAddr>>,
}

impl TransmissionChannelWorker {
    pub fn new(
        server_addr: SocketAddr,
        socket: Arc<UdpSocket>,
        addresses: Vec<SocketAddr>,
    ) -> Self {
        Self {
            socket,
            addresses: Mutex::new(addresses),
            server_addr,
        }
    }

    pub fn check_server_addr(&self, server_addr: SocketAddr) -> bool {
        return self.server_addr == server_addr;
    }

    pub fn add_client(&self, client: SocketAddr) {
        let mut lock = self.addresses.lock().unwrap();
        lock.push(client);
    }

    pub fn remove_client(&self, client: SocketAddr) {
        let mut lock = self.addresses.lock().unwrap();
        lock.retain(|&x| x != client);
    }

    pub fn run(&self) {
        println!("Listening on {}", self.socket.local_addr().unwrap());
        loop {
            let mut buffer_size = [0; 8]; 

            println!("Waiting for data...");
            self.socket
                .peek(&mut buffer_size)
                .expect("Error getting size of buffer");

            let size: u64 = bincode::deserialize(&buffer_size).expect("Error deserializing size");

            let mut buffer = vec![0; (size + 8) as usize];

            let n = self.socket
                .recv(&mut buffer)
                .expect("Error receiving packet");

            let buffer = &buffer[0..n];

            let addresses = self.addresses.lock().unwrap();

            for client in addresses.iter() {
                self.socket.send_to(buffer, client).unwrap();
            }
        }
    }
}
