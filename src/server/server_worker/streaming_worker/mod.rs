use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{IpAddr, TcpStream, UdpSocket},
    sync::{Arc, Mutex},
};

use rand::Rng;

use crate::{
    message::rtsp::{RequestType, RtspRequest, RtspResponse, Status},
    video::video_stream::VideoStream, server::server_worker::streaming_worker::video_stream_info::VideoStreamInfo,
};

pub mod transmission_worker;
pub mod video_stream_info;

use transmission_worker::TransmissionWorker;

#[derive(Debug, Clone, Copy, PartialEq)]
enum ServerState {
    Init,
    Ready,
    Playing,
}

#[derive(Debug)]
struct ClientInfo {
    ip_address: IpAddr,
    rtp_port: u16,
    session_id: u32,
}


#[derive(Debug)]
pub struct StreamingWorker<'a> {
    rtsp_socket: TcpStream,
    server_state: ServerState,
    client_info: Option<ClientInfo>,
    video_workers: &'a Mutex<HashMap<String, Arc<TransmissionWorker>>>,
}

impl<'a> StreamingWorker<'a> {
    pub fn new(
        rtsp_socket: TcpStream,
        video_workers: &'a Mutex<HashMap<String, Arc<TransmissionWorker>>>,
    ) -> Self {
        Self {
            rtsp_socket,
            server_state: ServerState::Init,
            client_info: None,
            video_workers,
        }
    }

    fn handle_client(&mut self, video_file: &str) {
        let mut lock = self.video_workers.lock().unwrap();

        let worker = lock.get(video_file);
        let client_info = self.client_info.as_ref().unwrap();

        let address = (client_info.ip_address, client_info.rtp_port);

        if let Some(worker) = worker {
            worker
                .add_client(address);
        } else {
            let addresses = vec![address];

            let stream = VideoStream::new(video_file);

            let video_info = Arc::new(VideoStreamInfo::new(stream, addresses));

            dbg!(&video_info);

            let rtp_socket = Arc::new(UdpSocket::bind("127.0.0.1:0").unwrap());

            lock.insert(
                video_file.to_string(),
                Arc::new(TransmissionWorker::new(rtp_socket, video_info)),
            );

            let worker = lock.get(video_file).unwrap();
            dbg!(&worker);

            let worker_clone = Arc::clone(&worker);

            std::thread::spawn(move || {
                worker_clone.run();
            });
        }
    }

    fn process_rtsp_request(&mut self, request: RtspRequest) {
        match request.request_type() {
            RequestType::Setup => {
                if let ServerState::Init = self.server_state {
                    println!("Processing setup");

                    let mut rng = rand::thread_rng();

                    let session_id = rng.gen_range(100000..999999);

                    self.client_info = Some(ClientInfo {
                        ip_address: self.rtsp_socket.peer_addr().unwrap().ip(),
                        rtp_port: request.port_rtp(),
                        session_id,
                    });

                    let response = RtspResponse::new(Status::Ok, request.seq_number(), session_id);

                    self.server_state = ServerState::Ready;

                    self.reply_rtsp(response);
                }
            }
            RequestType::Play => {
                if let ServerState::Ready = self.server_state {
                    self.process_play(request);
                }
            }
            RequestType::Teardown => {
                println!("Processing teardown");

                let response = RtspResponse::new(
                    Status::Ok,
                    request.seq_number(),
                    self.client_info.as_ref().unwrap().session_id,
                );

                self.reply_rtsp(response);
                let client_info = self.client_info.as_ref().unwrap();

                let address = (client_info.ip_address, client_info.rtp_port);

                let lock = self.video_workers.lock().unwrap();

                let worker = lock.get(request.file_request()).unwrap();

                worker
                    .remove_client(address);
            }
            _ => {
                todo!()
            }
        }
    }

    fn process_play(&mut self, request: RtspRequest) {
        println!("Processing play");
        let client_info = self.client_info.as_mut().unwrap();
        let session_id = client_info.session_id;

        self.server_state = ServerState::Playing;

        self.handle_client(request.file_request());

        let response = RtspResponse::new(Status::Ok, request.seq_number(), session_id);

        self.reply_rtsp(response);
    }

    pub fn reply_rtsp(&mut self, response: RtspResponse) {
        let response = bincode::serialize(&response).expect("Error serializing packet");

        self.rtsp_socket.write_all(&response).unwrap();
    }

    pub fn run(&mut self) {
        let mut buffer = [0; 1024];
        loop {
            let n = self.rtsp_socket.read(&mut buffer).unwrap();
            if n == 0 {
                continue;
            }

            let request = bincode::deserialize(&buffer).expect("Error deserializing packet");

            self.process_rtsp_request(request);
        }
    }
}
