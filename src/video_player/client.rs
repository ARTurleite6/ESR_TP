use std::{
    io::{Read, Write},
    net::{TcpStream, UdpSocket},
};

use thiserror::Error;

use crate::o_node::message::{
    self,
    rtp::RtpPacket,
    rtsp::{RequestType, RtspRequest, RtspResponse},
};

use super::{Args, VideoPlayerComponent};

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("Error requesting action")]
    FailedRequest,
    #[error("Action Not Possible at the moment, reason: {0}")]
    ActionNotPossible(String),
    #[error("Error connecting to server")]
    ConnectionError,
}

#[derive(Debug)]
struct ServerConnection {
    server_socket: TcpStream,
    udp_socket: Option<UdpSocket>,
    session_id: Option<u32>,
    stop_transmission: bool,
    sequence_number: u32,
}

#[derive(Debug, Default)]
pub struct Client {
    server_name: String,
    server_port: u16,
    rtp_port: u16,
    video_file: String,
    server_connection: Option<ServerConnection>,
}

impl VideoPlayerComponent for Client {
    type Init = Args;

    fn from_init(init: &Self::Init) -> Self {
        Self::new(
            init.server_name.clone(),
            init.server_port,
            init.rtp_port,
            init.video_file.clone(),
        )
    }
}

impl Client {
    pub fn new(server_name: String, server_port: u16, rtp_port: u16, video_file: String) -> Self {
        Self {
            server_name,
            server_port,
            rtp_port,
            video_file,
            ..Default::default()
        }
    }

    pub fn make_request(
        &mut self,
        request: RequestType,
        seq_number: u32,
    ) -> Result<RtspResponse, RequestError> {
        let server_connection =
            self.server_connection
                .as_mut()
                .ok_or(RequestError::ActionNotPossible(
                    "You must setup connection first".to_string(),
                ))?;

        let request = RtspRequest::new(request, self.video_file.clone(), seq_number, self.rtp_port);

        let request = bincode::serialize(&request).expect("Error serializing packet");

        let tcp_socket = &mut server_connection.server_socket;

        tcp_socket.write_all(&request).unwrap();

        let mut buffer = [0; 1024];

        tcp_socket.read(&mut buffer).unwrap();

        return Ok(bincode::deserialize(&buffer).expect("Error deserializing packet"));
    }

    pub fn session_id(&self) -> u32 {
        self.server_connection
            .as_ref()
            .expect("Expected server connection at this point")
            .session_id
            .expect("Expected session id at this point")
    }

    pub fn is_stopped(&self) -> bool {
        self.server_connection
            .as_ref()
            .expect("Expected server connection at this point")
            .stop_transmission
    }

    pub fn stop_transmition(&mut self) -> Result<(), RequestError> {
        let sequence_number = self
            .server_connection
            .as_ref()
            .ok_or(RequestError::ActionNotPossible(
                "No server connection".to_string(),
            ))?
            .sequence_number;

        let message = RtspRequest::new(
            message::rtsp::RequestType::Teardown,
            self.video_file.clone(),
            sequence_number + 1,
            self.rtp_port,
        );

        self.send_rtsp_packet(message);

        let response = self.receive_rtsp_packet();

        if !response.succeded() {
            return Err(RequestError::FailedRequest);
        }

        let server_connection =
            self.server_connection
                .as_mut()
                .ok_or(RequestError::ActionNotPossible(
                    "No server connection".to_string(),
                ))?;

        server_connection.stop_transmission = true;
        server_connection.udp_socket = None;

        Ok(())
    }

    pub fn setup(&mut self) -> Result<(), RequestError> {
        let seq_number = 1;

        let message = RtspRequest::new(
            message::rtsp::RequestType::Setup,
            self.video_file.clone(),
            seq_number,
            self.rtp_port,
        );

        let udp_socket =
            UdpSocket::bind(("127.0.0.1", self.rtp_port)).expect("Error binding rtp socket");

        let server_socket = TcpStream::connect((self.server_name.as_str(), self.server_port))
            .or_else(|_| Err(RequestError::FailedRequest))?;

        self.server_connection = Some(ServerConnection {
            server_socket,
            udp_socket: Some(udp_socket),
            session_id: None,
            stop_transmission: false,
            sequence_number: 1,
        });

        self.send_rtsp_packet(message);

        let response = self.receive_rtsp_packet();

        self.server_connection.as_mut().unwrap().session_id = Some(response.session_id());

        return Ok(());
    }

    pub fn receive_rtp_packet(&self) -> RtpPacket {
        let mut buffer_size = [0; 8];

        let udp_socket = &self
            .server_connection
            .as_ref()
            .unwrap()
            .udp_socket
            .as_ref()
            .expect("Error getting udp socket");

        udp_socket
            .peek(&mut buffer_size)
            .expect("Error geetting size of buffer");

        let size: u64 = bincode::deserialize(&buffer_size).expect("Error deserializing size");

        let mut buffer = vec![0; (size + 8) as usize];

        let n = udp_socket
            .recv(&mut buffer)
            .expect("Error receiving packet");

        dbg!(buffer.len());
        let buffer = &buffer[8..n];
        dbg!(buffer.len());

        return RtpPacket::decode(buffer);
    }

    fn send_rtsp_packet(&mut self, packet: RtspRequest) {
        self.server_connection
            .as_mut()
            .unwrap()
            .server_socket
            .write_all(&bincode::serialize(&packet).expect("Error serializing packet"))
            .expect("Error sending packet to server");
    }

    fn receive_rtsp_packet(&mut self) -> RtspResponse {
        let mut buffer = [0; 1024];

        self.server_connection
            .as_mut()
            .unwrap()
            .server_socket
            .read(&mut buffer)
            .expect("Error receiving packet from server");

        return bincode::deserialize(&buffer).expect("Error deserializing packet");
    }
}
