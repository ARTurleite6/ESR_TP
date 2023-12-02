use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream, UdpSocket},
    sync::{Arc, Mutex},
};

use crate::{
    message::rtsp::RtspRequest,
    video::{packet_source::PacketSource, video_stream::VideoStream},
};

#[derive(Debug, PartialEq)]
pub struct ClientInfo {
    address: SocketAddr,
    session_id: u32,
}

impl ClientInfo {
    pub fn new(address: SocketAddr, session_id: u32) -> Self {
        Self {
            address,
            session_id,
        }
    }

    pub fn session_id(&self) -> u32 {
        return self.session_id;
    }

    pub fn address(&self) -> &SocketAddr {
        &self.address
    }
}

#[derive(Debug)]
pub struct TransmissionChannel {
    server_stream: TcpStream,
    udp_socket: Arc<UdpSocket>,
    clients: Vec<ClientInfo>,
    worker: Option<Arc<TransmissionChannelWorker>>,
}

impl TransmissionChannel {
    pub fn new(
        server_stream: TcpStream,
        udp_socket: Arc<UdpSocket>,
        clients: Vec<ClientInfo>,
    ) -> Self {
        Self {
            server_stream: server_stream.into(),
            udp_socket,
            clients,
            worker: None,
        }
    }

    pub fn create_worker(&mut self, client: ClientInfo) {
        let socket_clone = Arc::clone(&self.udp_socket);

        let worker = Arc::new(TransmissionChannelWorker::new(
            socket_clone,
            vec![client.address],
        ));

        self.worker = Some(worker);

        let worker_clone = Arc::clone(&self.worker.as_ref().unwrap());

        std::thread::spawn(move || {
            worker_clone.run();
        });
    }

    pub fn send_server_request(&mut self, request: RtspRequest) -> std::io::Result<Vec<u8>> {
        self.server_stream
            .write(&bincode::serialize(&request).unwrap())?;

        let mut buffer = [0; 1024];

        let n = self.server_stream.read(&mut buffer)?;
        let answer: Vec<u8> = buffer[..n].to_vec();
        return Ok(answer);
    }

    pub fn add_client_to_room(&mut self, client: ClientInfo) {
        self.clients.push(client);
    }

    pub fn remove_client_to_room(&mut self, client: ClientInfo) {
        self.clients.retain(|cl| cl != &client);
    }

    pub fn add_client_as_playable(&mut self, client: ClientInfo) {
        let address = client.address;
        self.clients.push(client);
        self.worker.as_ref().unwrap().add_client(address);
    }

    pub fn remove_client_as_playable(&mut self, client: ClientInfo) {
        self.clients.retain(|client| client != client);
        self.worker.as_ref().unwrap().remove_client(client.address);
    }

    pub fn has_clients(&self) -> bool {
        return !self.clients.is_empty();
    }

    pub fn has_worker(&self) -> bool {
        return self.worker.is_some();
    }

    pub fn rtp_port(&self) -> u16 {
        return self.udp_socket.local_addr().unwrap().port();
    }
}

#[derive(Debug)]
pub struct TransmissionChannelWorker {
    socket: Arc<UdpSocket>,
    video_stream: Option<Mutex<VideoStream>>,
    addresses: Mutex<Vec<SocketAddr>>,
}

impl TransmissionChannelWorker {
    pub fn new(socket: Arc<UdpSocket>, addresses: Vec<SocketAddr>) -> Self {
        Self {
            socket,
            addresses: Mutex::new(addresses),
            video_stream: None,
        }
    }

    pub fn with_video_stream(
        socket: Arc<UdpSocket>,
        addresses: Vec<SocketAddr>,
        video_stream: VideoStream,
    ) -> Self {
        Self {
            socket,
            addresses: Mutex::new(addresses),
            video_stream: Some(Mutex::new(video_stream)),
        }
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
            let packet = if let Some(video_stream) = &self.video_stream {
                let mut lock = video_stream.lock().unwrap();
                lock.receive_next_packet()
            } else {
                self.socket.receive_next_packet()
            };

            if let Ok(packet) = packet {
                let addresses = self.addresses.lock().unwrap();
                dbg!(&addresses);
                for client in addresses.iter() {
                    self.socket.send_to(&packet, client).unwrap();
                }
            } else {
                println!("Error receiving packet");
            }
        }
    }
}
