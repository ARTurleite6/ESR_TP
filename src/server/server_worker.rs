use std::{
    io::{Read, Write},
    net::{IpAddr, TcpStream, UdpSocket},
    str::FromStr, sync::{Arc, atomic::AtomicBool},
};

use rand::Rng;

use crate::{
    o_node::message::rtsp::{RequestType, RtspRequest, RtspResponse, Status},
    video::video_stream::VideoStream,
};

use super::Server;

#[derive(Debug, Clone, Copy, PartialEq)]
enum ServerState {
    Init,
    Ready,
    Playing,
}

#[derive(Debug)]
pub struct ServerWorker {
    rtsp_socket: TcpStream,
    server_state: ServerState,
    client_info: Option<ClientInfo>,
    stop_transmission: Arc<AtomicBool>,
}

#[derive(Debug)]
struct ClientInfo {
    ip_address: IpAddr,
    rtp_port: u16,
    video_stream: VideoStream,
    session_id: u32,
    socket_rtp: Option<UdpSocket>,
}

impl ClientInfo {
    fn open_connection(&mut self) {
        let socket_rtp = UdpSocket::bind("127.0.0.1:0").expect("Error binding socket");
        self.socket_rtp = Some(socket_rtp);
    }
}

impl ServerWorker {
    pub fn new(rtsp_socket: TcpStream) -> Self {
        Self {
            rtsp_socket,
            server_state: ServerState::Init,
            client_info: None,
            stop_transmission: Arc::new(AtomicBool::new(true)),
        }
    }

    fn process_rtsp_request(&mut self, request: RtspRequest) {
        match request.request_type() {
            RequestType::Setup => {
                if let ServerState::Init = self.server_state {
                    println!("Processing setup");

                    let video_stream = VideoStream::new(request.file_request());

                    let mut rng = rand::thread_rng();

                    let session_id = rng.gen_range(100000..999999);

                    self.client_info = Some(ClientInfo {
                        ip_address: self.rtsp_socket.peer_addr().unwrap().ip(),
                        rtp_port: request.port_rtp(),
                        video_stream,
                        session_id,
                        socket_rtp: None,
                    });
                    dbg!(&self.client_info);

                    let response = RtspResponse::new(Status::Ok, request.seq_number(), session_id);

                    self.server_state = ServerState::Ready;

                    self.reply_rtsp(response);
                }
            }
            RequestType::Play => {
                if let ServerState::Ready = self.server_state {
                    println!("Processing play");
                    self.client_info.as_mut().unwrap().open_connection();
                }
            }
            _ => {
                todo!()
            }
        }
    }

    pub fn reply_rtsp(&mut self, response: RtspResponse) {
        let response = bincode::serialize(&response).expect("Error serializing packet");

        dbg!(&response);

        self.rtsp_socket.write_all(&response).unwrap();
    }

    pub fn run(&mut self) {
        let mut buffer = [0; 1024];
        loop {
            let n = self.rtsp_socket.read(&mut buffer).unwrap();
            if n == 0 {
                continue;
            }
            dbg!(&buffer);

            let request = bincode::deserialize(&buffer).expect("Error deserializing packet");

            self.process_rtsp_request(request);
        }
    }
}
