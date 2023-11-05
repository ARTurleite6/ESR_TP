use std::{
    io::{Read, Write},
    net::TcpStream,
    str::FromStr,
};

use rand::Rng;

use crate::{
    o_node::message::rtsp::{RequestType, RtspRequest, RtspResponse, Status},
    video::video_stream::VideoStream,
};

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
}

#[derive(Debug)]
struct ClientInfo {
    rtp_port: u16,
    video_stream: VideoStream,
    session_id: u32,
}

impl ServerWorker {
    pub fn new(rtsp_socket: TcpStream) -> Self {
        Self {
            rtsp_socket,
            server_state: ServerState::Init,
            client_info: None,
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
                        rtp_port: request.port_rtp(),
                        video_stream,
                        session_id,
                    });
                    dbg!(&self.client_info);

                    let response = RtspResponse::new(Status::Ok, request.seq_number(), session_id);

                    self.server_state = ServerState::Ready;

                    self.reply_rtsp(response);
                }
            }
            _ => {
                todo!()
            }
        }
    }

    pub fn reply_rtsp(&mut self, response: RtspResponse) {
        let response: String = response.to_string();

        dbg!(&response);

        self.rtsp_socket.write_all(response.as_bytes()).unwrap();
    }

    pub fn run(&mut self) {
        let mut buffer = [0; 1024];
        loop {
            let n = self.rtsp_socket.read(&mut buffer).unwrap();
            if n == 0 {
                continue;
            }
            dbg!(&buffer);

            let buffer = String::from_utf8_lossy(&buffer[..n]);

            let request = RtspRequest::from_str(&buffer).unwrap();

            self.process_rtsp_request(request);
        }
    }
}
