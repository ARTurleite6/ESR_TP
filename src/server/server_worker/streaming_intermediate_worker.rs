use std::{sync::{Mutex, Arc}, net::{TcpStream, TcpListener, IpAddr, SocketAddr, UdpSocket}, collections::HashMap, io::{Read, Write}};

use crate::{message::rtsp::{RtspRequest, RequestType, RtspResponse}, server::transmission_channel::TransmissionChannelWorker};

#[derive(Debug)]
pub struct TransmissionChannel {
    video: String,
    server_address: SocketAddr,
    server_stream: TcpStream,
    udp_socket: Arc<UdpSocket>,
    clients: Vec<ClientInfo>,
    worker: Option<Arc<TransmissionChannelWorker>>,
}

#[derive(Debug)]
struct ClientInfo {
    ip_address: IpAddr,
    rtp_port: u16,
    session_id: u32,
}

pub struct StreamingWorker<'a> {
    port: u16,
    transmission_workers: &'a Mutex<HashMap<String, TransmissionChannel>>,
}

impl StreamingWorker<'_> {
    pub fn new(port: u16, transmission_workers: &Mutex<HashMap<String, TransmissionChannel>>) -> StreamingWorker {
        StreamingWorker {
            port,
            transmission_workers,
        }
    }

    fn streaming_service_worker(&self, mut stream: TcpStream) {
        loop {
            let mut buffer = [0; 1024];
            let n = stream.read(&mut buffer).unwrap();
            if n == 0 {
                continue;
            }

            let message: RtspRequest = bincode::deserialize(&buffer[..n]).unwrap();

            let address = stream.peer_addr().unwrap();

            let client_info = ClientInfo {
                ip_address: address.ip(),
                rtp_port: message.port_rtp(),
                session_id: message.seq_number(),
            };

            match message.request_type() {
                RequestType::Setup => {
                    self.process_setup(&mut stream, message);
                }
                RequestType::Play => self.process_play(&mut stream, message, client_info),
                RequestType::Teardown => {}
                _ => todo!(),
            }
        }
    }

    fn process_play(&self, stream: &mut TcpStream, request: RtspRequest, address: ClientInfo) {
        let mut lock_guard = self.transmission_workers.lock().unwrap();

        let transmission_worker = lock_guard
            .get_mut(request.file_request())
            .expect("Expected Connection Channel");

        let port = transmission_worker.udp_socket.local_addr().unwrap().port();

        let request = RtspRequest::new(
            RequestType::Play,
            request.file_request().to_string(),
            request.seq_number(),
            port,
        );

        if let Some(worker) = &transmission_worker.worker {
            worker.add_client(SocketAddr::new(address.ip_address, address.rtp_port));

            let answer = RtspResponse::new(
                crate::message::rtsp::Status::Ok,
                request.seq_number(),
                request.seq_number(),
            );
            stream.write(&bincode::serialize(&answer).unwrap()).unwrap();
        } else {
            let socket_clone = Arc::clone(&transmission_worker.udp_socket);

            let worker = Arc::new(TransmissionChannelWorker::new(
                transmission_worker.server_address,
                socket_clone,
                vec![SocketAddr::new(address.ip_address, address.rtp_port)],
            ));

            transmission_worker.worker = Some(worker);

            let worker_clone = Arc::clone(&transmission_worker.worker.as_ref().unwrap());

            std::thread::spawn(move || {
                worker_clone.run();
            });

            transmission_worker
                .server_stream
                .write(&bincode::serialize(&request).unwrap())
                .unwrap();

            let mut buffer = [0; 1024];
            let n = transmission_worker.server_stream.read(&mut buffer).unwrap();

            stream.write(&buffer[..n]).unwrap();
        }
    }

    fn process_setup(&self, client_stream: &mut TcpStream, mut request: RtspRequest) {
        let mut lock_guard = self.transmission_workers.lock().unwrap();

        let worker = lock_guard.get_mut(request.file_request());

        if let Some(worker) = worker {
            worker.clients.push(ClientInfo {
                ip_address: client_stream.local_addr().unwrap().ip(),
                rtp_port: request.port_rtp(),
                session_id: request.seq_number(),
            });

            let answer = RtspResponse::new(
                crate::message::rtsp::Status::Ok,
                request.seq_number(),
                request.seq_number(),
            );

            client_stream
                .write(&bincode::serialize(&answer).unwrap())
                .unwrap();

            dbg!(worker.clients.len());
        } else {
            let server_to_contact = request.next_server().expect("Expected server to contact");
            dbg!(&server_to_contact);
            let mut server_stream = TcpStream::connect(server_to_contact.address()).unwrap();

            let udp_socket = Arc::new(UdpSocket::bind(("0.0.0.0", 0)).unwrap());

            let request_server = RtspRequest::new_with_servers(
                RequestType::Setup,
                request.file_request().to_string(),
                request.seq_number(),
                udp_socket.local_addr().unwrap().port(),
                request.servers_to_connect().clone(),
            );

            let client_info = ClientInfo {
                ip_address: client_stream.peer_addr().unwrap().ip(),
                rtp_port: request.port_rtp(),
                session_id: request.seq_number(),
            };

            server_stream
                .write(&bincode::serialize(&request_server).unwrap())
                .unwrap();

            let mut buffer = [0; 1024];

            let n = server_stream.read(&mut buffer).unwrap();
            let answer: RtspResponse = bincode::deserialize(&buffer[..n]).unwrap();

            client_stream.write(&buffer[..n]).unwrap();

            let channel = TransmissionChannel {
                video: request.file_request().to_string(),
                server_stream,
                udp_socket,
                worker: None,
                clients: vec![client_info],
                server_address: server_to_contact.address().into(),
            };

            lock_guard.insert(request.file_request().to_string(), channel);
        }
    }
    
    pub fn run(&self) {
        std::thread::scope(|s| {
            let tcp_socket = TcpListener::bind(("0.0.0.0", self.port)).unwrap();
            println!(
                "Streaming service listening on port {}",
                tcp_socket.local_addr().unwrap().port()
            );

            for stream in tcp_socket.incoming() {
                let stream = stream.unwrap();
                s.spawn(move || {
                    self.streaming_service_worker(stream);
                });
            }
        });
    }
}