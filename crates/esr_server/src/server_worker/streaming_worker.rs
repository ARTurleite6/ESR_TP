use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{IpAddr, TcpStream, UdpSocket},
    sync::mpsc::{self, Sender},
};

use esr_core::message::rtsp::{RequestType, RtspRequest, RtspResponse, Status};
use rand::Rng;

use esr_video::video_stream::VideoStream;

use crate::server_worker::streaming_worker::video_stream_info::VideoStreamInfo;

pub mod transmission_worker;
pub mod video_stream_info;

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

pub enum Message<T> {
    Add(T),
    Remove(T),
}

type StreamingMessage = Message<(IpAddr, u16)>;

#[derive(Debug)]
pub struct StreamingWorker {
    rtsp_socket: TcpStream,
    server_state: ServerState,
    client_info: Option<ClientInfo>,
    video_workers: HashMap<String, Sender<StreamingMessage>>,
    sender: mpsc::Sender<Message<String>>,
}

impl StreamingWorker {
    pub fn new(rtsp_socket: TcpStream, sender: mpsc::Sender<Message<String>>) -> Self {
        Self {
            rtsp_socket,
            server_state: ServerState::Init,
            client_info: None,
            video_workers: Default::default(),
            sender,
        }
    }

    fn handle_client(&mut self, video_file: &str) -> std::io::Result<()> {
        let worker = self.video_workers.get(video_file);
        let client_info = self.client_info.as_ref().unwrap();

        let address = (client_info.ip_address, client_info.rtp_port);

        if let Some(worker) = worker {
            worker.send(StreamingMessage::Add(address)).unwrap();
        } else {
            let addresses = vec![address];

            let stream = VideoStream::new(video_file)?;

            let video_info = VideoStreamInfo::new(stream, addresses);

            let rtp_socket = UdpSocket::bind("0.0.0.0:0")?;

            let (sender, receiver) = mpsc::channel();
            std::thread::spawn(move || {
                transmission_worker::run(rtp_socket, video_info, receiver);
            });

            self.video_workers.insert(video_file.to_string(), sender);
        }
        Ok(())
    }

    fn process_rtsp_request(&mut self, request: RtspRequest) -> std::io::Result<()> {
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

                    if !VideoStream::file_exists(request.file_request()) {
                        let response = RtspResponse::new(
                            Status::FileNotFound,
                            request.seq_number(),
                            session_id,
                        );

                        self.client_info = None;
                        self.reply_rtsp(response)?;
                    }

                    let response = RtspResponse::new(Status::Ok, request.seq_number(), session_id);

                    self.server_state = ServerState::Ready;

                    self.reply_rtsp(response)?;
                }
            }
            RequestType::Play => {
                if let ServerState::Ready = self.server_state {
                    self.process_play(request)?;
                }
            }
            RequestType::Teardown => {
                println!("Processing teardown");

                let client_info = self.client_info.as_ref().unwrap();

                let response =
                    RtspResponse::new(Status::Ok, request.seq_number(), client_info.session_id);

                let address = (client_info.ip_address, client_info.rtp_port);

                let worker = self.video_workers.get(request.file_request()).unwrap();

                if worker.send(StreamingMessage::Remove(address)).is_ok() {
                    println!("Removing worker");
                    self.video_workers.remove(request.file_request()).unwrap();
                    self.sender
                        .send(Message::Remove(request.file_request().to_string()))
                        .unwrap();
                }

                self.reply_rtsp(response)?;
            }
            RequestType::Pause => {
                println!("Processing Pause");

                let client_info = self.client_info.as_ref().unwrap();

                let response =
                    RtspResponse::new(Status::Ok, request.seq_number(), client_info.session_id);

                let address = (client_info.ip_address, client_info.rtp_port);

                let worker = self.video_workers.get(request.file_request()).unwrap();
                worker.send(StreamingMessage::Remove(address)).unwrap();

                self.reply_rtsp(response)?;
            }
        }
        Ok(())
    }

    fn process_play(&mut self, request: RtspRequest) -> std::io::Result<()> {
        println!("Processing play");
        let client_info = self.client_info.as_mut().unwrap();
        let session_id = client_info.session_id;

        self.server_state = ServerState::Playing;

        if self.handle_client(request.file_request()).is_err() {
            let response =
                RtspResponse::new(Status::ConnectionError, request.seq_number(), session_id);

            return self.reply_rtsp(response);
        }

        let response = RtspResponse::new(Status::Ok, request.seq_number(), session_id);

        self.reply_rtsp(response)
    }

    pub fn reply_rtsp(&mut self, response: RtspResponse) -> std::io::Result<()> {
        let response = bincode::serialize(&response).expect("Error serializing packet");

        self.rtsp_socket.write_all(&response)
    }

    pub fn run(&mut self) {
        let mut buffer = [0; 1024];
        loop {
            let n = self.rtsp_socket.read(&mut buffer).unwrap();
            if n == 0 {
                continue;
            }

            let request = bincode::deserialize(&buffer).expect("Error deserializing packet");

            match self.process_rtsp_request(request) {
                Ok(_) => {
                    println!("Request processed successfully")
                }
                Err(error) => {
                    println!("Error processing request {}", error);
                }
            }
        }
    }
}
