use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream, UdpSocket},
    sync::{Arc, Mutex},
};

use rand::Rng;

use crate::{
    message::rtsp::{RequestType, RtspRequest, RtspResponse},
    server::transmission_channel::{ClientInfo, TransmissionChannel},
};
#[derive(Debug)]
pub struct StreamingWorker<'a> {
    port: u16,
    transmission_workers: &'a Mutex<HashMap<String, TransmissionChannel>>,
}

impl StreamingWorker<'_> {
    pub fn new(
        port: u16,
        transmission_workers: &Mutex<HashMap<String, TransmissionChannel>>,
    ) -> StreamingWorker {
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
            dbg!(&message);
            dbg!(&stream);

            let answer = match message.request_type() {
                RequestType::Setup => self.process_setup(&mut stream, message),
                RequestType::Play => self.process_play(&mut stream, message),
                RequestType::Teardown => todo!(),
                _ => todo!(),
            };

            stream.write(&answer).unwrap();
        }
    }

    fn process_play(&self, stream: &mut TcpStream, request: RtspRequest) -> Vec<u8> {
        let mut lock_guard = self.transmission_workers.lock().unwrap();

        let transmission_worker = lock_guard
            .get_mut(request.file_request())
            .expect("Expected Connection Channel");

        let port = transmission_worker.rtp_port();

        let address = ClientInfo::new(
            SocketAddr::new(stream.peer_addr().unwrap().ip(), request.port_rtp()),
            request.seq_number(),
        );

        if transmission_worker.has_worker() {
            transmission_worker.add_client_as_playable(address);

            let answer = RtspResponse::new(
                crate::message::rtsp::Status::Ok,
                request.seq_number(),
                request.seq_number(),
            );
            return bincode::serialize(&answer).unwrap();
        } else {

            let request = RtspRequest::new(
                RequestType::Play,
                request.file_request().to_string(),
                request.seq_number(),
                port,
            );

            let answer = transmission_worker.send_server_request(request).unwrap();
            let answer_decode: RtspResponse = bincode::deserialize(&answer).unwrap();

            if !answer_decode.succeded() {
                return answer;
            }

            transmission_worker.create_worker(address);

            return answer;
        }
    }

    fn process_setup(&self, client_stream: &mut TcpStream, mut request: RtspRequest) -> Vec<u8> {
        let mut lock_guard = self.transmission_workers.lock().unwrap();

        let channel = lock_guard.get_mut(request.file_request());

        if let Some(channel) = channel {
            let session_id = rand::thread_rng().gen();

            channel.add_client_to_room(ClientInfo::new(
                SocketAddr::new(client_stream.peer_addr().unwrap().ip(), request.port_rtp()),
                session_id,

            ));

            let answer = RtspResponse::new(
                crate::message::rtsp::Status::Ok,
                request.seq_number(),
                request.seq_number(),
            );

            dbg!(&channel);
            return bincode::serialize(&answer).unwrap();
        } else {
            let server_to_contact = request.next_server().expect("Expected server to contact");
            dbg!(&server_to_contact);
            let server_stream = TcpStream::connect(server_to_contact.address()).unwrap();

            let udp_socket = Arc::new(UdpSocket::bind(("0.0.0.0", 0)).unwrap());

            let port = udp_socket.local_addr().unwrap().port();

            let mut channel = TransmissionChannel::new(server_stream, udp_socket, vec![]);


            let request_server = RtspRequest::new_with_servers(
                RequestType::Setup,
                request.file_request().to_string(),
                request.seq_number(),
                port,
                request.servers_to_connect().clone(),
            );


            let answer = channel.send_server_request(request_server).unwrap();

            let answer_decode: RtspResponse = bincode::deserialize(&answer).unwrap();

            if !answer_decode.succeded() {
                return answer;
            }

            let client_info = ClientInfo::new(
                SocketAddr::new(client_stream.peer_addr().unwrap().ip(), request.port_rtp()),
                answer_decode.session_id(),
            );

            channel.add_client_to_room(client_info);


            let answer = channel.send_server_request(request_server).unwrap();
            lock_guard.insert(request.file_request().to_string(), channel);
            drop(lock_guard);

            return answer;
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
