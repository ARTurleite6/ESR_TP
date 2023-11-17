use std::{
    io::{Read, Write},
    net::{IpAddr, TcpStream, UdpSocket},
    sync::{atomic::AtomicBool, Arc, Mutex},
    time::Duration,
};

use rand::Rng;

use crate::{
    message::{
        rtp::RtpPacketBuilder,
        rtsp::{RequestType, RtspRequest, RtspResponse, Status},
    },
    video::video_stream::VideoStream,
};

struct VideoStreamInfo {
    video_stream: VideoStream,
    clients: Vec<(IpAddr, u16)>,
}

impl VideoStreamInfo {
    pub fn new(video_stream: VideoStream, clients: Vec<(IpAddr, u16)>) -> Self {
        Self {
            video_stream,
            clients: Vec::new(),
        }
    }

    fn next_frame(&mut self) -> std::io::Result<Vec<u8>> {
        self.video_stream.next_frame()
    }

    fn frame_num(&self) -> u32 {
        self.video_stream.frame_num()
    }

    fn send_data(&self, data: &[u8], rtp_socket: &UdpSocket) {
        for client in &self.clients {
            rtp_socket
                .send_to(data, client)
                .expect("Error sending data");
        }
    }

    fn add_client(&mut self, client: (IpAddr, u16)) {
        self.clients.push(client);
    }

    fn remove_client(&mut self, client: (IpAddr, u16)) {
        self.clients.retain(|c| c != &client);
    }
}

#[derive(Debug)]
pub struct TransmissionWorker {
    rtp_socket: Arc<UdpSocket>,
    video_client_addrs: Arc<Mutex<VideoStreamInfo>>,
}

impl TransmissionWorker {
    pub fn new(
        rtp_socket: Arc<UdpSocket>,
        video_client_addrs: Arc<Mutex<VideoStreamInfo>>,
    ) -> Self {
        Self {
            rtp_socket,
            video_client_addrs,
        }
    }

    fn run(&self, stop_transmiting: Arc<AtomicBool>) {
        loop {
            std::thread::sleep(Duration::from_secs_f64(0.05));

            if stop_transmiting.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }

            let mut lock_guard = self.video_client_addrs.lock().unwrap();

            let data = lock_guard.next_frame();

            if let Ok(data) = data {
                let frame_number = lock_guard.frame_num();

                let packet = RtpPacketBuilder::new(&data, 26)
                    .sequence_number(frame_number as u16)
                    .build();

                let encode = packet.transmit_data();

                let size = encode.len() as u64;
                dbg!(size);

                let size_encoded = bincode::serialize(&size).expect("Error serializing size");

                let mut encoded = size_encoded;
                encoded.extend(encode);

                lock_guard.send_data(&encoded, &self.rtp_socket)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ServerState {
    Init,
    Ready,
    Playing,
}

#[derive(Debug)]
pub struct StreamingWorker {
    rtsp_socket: TcpStream,
    server_state: ServerState,
    client_info: Option<ClientInfo>,
    stop_transmission: Arc<AtomicBool>,
}

#[derive(Debug)]
struct ClientInfo {
    ip_address: IpAddr,
    rtp_port: u16,
    video_stream: Arc<Mutex<VideoStream>>,
    session_id: u32,
    socket_rtp: Option<Arc<UdpSocket>>,
}

impl ClientInfo {
    fn open_connection(&mut self) {
        let socket_rtp = Arc::new(UdpSocket::bind("127.0.0.1:0").expect("Error binding socket"));
        self.socket_rtp = Some(socket_rtp);
    }

    fn close_connection(&mut self) {
        self.socket_rtp = None;
    }
}

impl StreamingWorker {
    pub fn new(rtsp_socket: TcpStream) -> Self {
        Self {
            rtsp_socket,
            server_state: ServerState::Init,
            client_info: None,
            stop_transmission: Arc::new(AtomicBool::new(false)),
        }
    }

    fn process_rtsp_request(&mut self, request: RtspRequest) {
        match request.request_type() {
            RequestType::Setup => {
                if let ServerState::Init = self.server_state {
                    println!("Processing setup");

                    let video_stream =
                        Arc::new(Mutex::new(VideoStream::new(request.file_request())));

                    let mut rng = rand::thread_rng();

                    let session_id = rng.gen_range(100000..999999);

                    self.client_info = Some(ClientInfo {
                        ip_address: self.rtsp_socket.peer_addr().unwrap().ip(),
                        rtp_port: request.port_rtp(),
                        video_stream,
                        session_id,
                        socket_rtp: None,
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

                self.stop_transmission
                    .store(true, std::sync::atomic::Ordering::SeqCst);

                let response = RtspResponse::new(
                    Status::Ok,
                    request.seq_number(),
                    self.client_info.as_ref().unwrap().session_id,
                );

                self.reply_rtsp(response);

                self.client_info
                    .as_mut()
                    .expect("Expected client information")
                    .close_connection();
            }
            _ => {
                todo!()
            }
        }
    }

    fn process_play(&mut self, request: RtspRequest) {
        println!("Processing play");
        let client_info = self.client_info.as_mut().unwrap();
        client_info.open_connection();

        self.server_state = ServerState::Playing;

        let socket_clone = Arc::clone(
            &client_info
                .socket_rtp
                .as_ref()
                .expect("Expected socket connection"),
        );

        let stop_transmiting_clone = Arc::clone(&self.stop_transmission);

        let video_stream_clone = Arc::clone(&client_info.video_stream);
        let ip_address = client_info.ip_address;
        let rtp_port = client_info.rtp_port;
        std::thread::spawn(move || {
            TransmissionWorker::run(
                video_stream_clone,
                socket_clone,
                stop_transmiting_clone,
                (ip_address, rtp_port),
            );
        });

        let response = RtspResponse::new(Status::Ok, request.seq_number(), client_info.session_id);

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
