use std::{
    io::{Read, Write},
    net::{TcpStream, UdpSocket},
};

use thiserror::Error;

use crate::{
    message::{
        self,
        answer::Answer,
        query::Query,
        rtp::RtpPacket,
        rtsp::{RequestType, RtspRequest, RtspResponse},
        Message,
    },
    o_node::neighbour::Neighbour,
};

use super::{Args, VideoPlayerComponent};

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("Error requesting action")]
    FailedRequest,
    #[error("Action Not Possible at the moment, reason: {0}")]
    ActionNotPossible(String),
    #[error("Error connecting to server{0}")]
    ConnectionError(String),
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
    servers_to_connect: Vec<Neighbour>,
}

impl VideoPlayerComponent for Client {
    type Init = Args;

    fn from_init(init: &Self::Init) -> Self {
        dbg!(init);
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

    pub fn make_request(&mut self, request: RequestType) -> Result<RtspResponse, RequestError> {
        let server_connection =
            self.server_connection
                .as_mut()
                .ok_or(RequestError::ActionNotPossible(
                    "You must setup connection first".to_string(),
                ))?;

        let seq_number = server_connection.sequence_number;

        let request = RtspRequest::new_with_servers(
            request,
            self.video_file.clone(),
            seq_number,
            self.rtp_port,
            self.servers_to_connect.clone(),
        );

        dbg!(&request);

        let request = bincode::serialize(&request).expect("Error serializing packet");

        let tcp_socket = &mut server_connection.server_socket;

        let n = tcp_socket
            .write(&request)
            .map_err(|err| RequestError::ConnectionError(err.to_string()))?;

        dbg!(n);

        let mut buffer = [0; 1024];

        tcp_socket
            .read(&mut buffer)
            .map_err(|err| RequestError::ConnectionError(err.to_string()))?;

        Ok(bincode::deserialize(&buffer).expect("Error deserializing packet"))
    }

    pub fn session_id(&self) -> Option<u32> {
        self.server_connection.as_ref()?.session_id
    }

    pub fn is_stopped(&self) -> Option<bool> {
        self.server_connection.as_ref()?.stop_transmission.into()
    }

    pub fn stop_transmition(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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

        self.send_rtsp_packet(message)?;

        let response = self.receive_rtsp_packet()?;

        if !response.succeded() {
            return Err(RequestError::FailedRequest.into());
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

    pub fn find_video(
        &self,
        udp_socket: &UdpSocket,
    ) -> Result<Answer<Vec<Neighbour>>, RequestError> {
        let query = Query::new_file_query(&self.video_file, None);

        let query_encode = bincode::serialize(&query).unwrap();

        dbg!(&(self.server_name.as_str(), self.server_port));
        let n = udp_socket
            .send_to(&query_encode, (self.server_name.as_str(), self.server_port))
            .map_err(|err| RequestError::ConnectionError(err.to_string()))?;
        dbg!(n);

        let mut buffer = [0; 1024];
        let n = udp_socket
            .recv(&mut buffer)
            .map_err(|err| RequestError::ConnectionError(err.to_string()))?;
        let answer = bincode::deserialize(&buffer[..n]).unwrap();

        dbg!(&answer);

        Ok(answer)
    }

    pub fn setup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let udp_socket = UdpSocket::bind(("0.0.0.0", self.rtp_port))
            .map_err(|err| RequestError::ConnectionError(err.to_string()))?;

        let answer = self.find_video(&udp_socket)?;
        dbg!(&answer);

        if !answer.status().is_ok() {
            return Err(RequestError::FailedRequest.into());
        }

        let servers_to_connect = answer.payload().ok_or(RequestError::ConnectionError(
            "Expected payload on the answer".to_string(),
        ))?;
        self.servers_to_connect = servers_to_connect.clone();
        dbg!(&self.servers_to_connect);

        let seq_number = 1;

        let message = RtspRequest::new_with_servers(
            message::rtsp::RequestType::Setup,
            self.video_file.clone(),
            seq_number,
            self.rtp_port,
            servers_to_connect.clone(),
        );

        let server_socket = TcpStream::connect((self.server_name.as_str(), self.server_port))
            .map_err(|_| RequestError::FailedRequest)?;

        self.server_connection = Some(ServerConnection {
            server_socket,
            udp_socket: Some(udp_socket),
            session_id: None,
            stop_transmission: false,
            sequence_number: 1,
        });

        self.send_rtsp_packet(message)?;

        let response = self.receive_rtsp_packet()?;

        dbg!(&response);
        if !response.succeded() {
            self.server_connection = None;
            return Err(RequestError::FailedRequest.into());
        }

        self.server_connection
            .as_mut()
            .ok_or(RequestError::ActionNotPossible(
                "Client must have a connection with server".to_string(),
            ))?
            .session_id = Some(response.session_id());

        Ok(())
    }

    pub fn receive_rtp_packet(&self) -> Result<RtpPacket, RequestError> {
        let mut buffer_size = [0; 8];

        let udp_socket = &self
            .server_connection
            .as_ref()
            .ok_or(RequestError::ActionNotPossible(
                "Client must have a connection with server".to_string(),
            ))?
            .udp_socket
            .as_ref()
            .ok_or(RequestError::ActionNotPossible(
                "Client must have a connection with server".to_string(),
            ))?;

        udp_socket.peek(&mut buffer_size).map_err(|_| {
            RequestError::ActionNotPossible("Client must have a connection with server".to_string())
        })?;

        let size: u64 = bincode::deserialize(&buffer_size).expect("Error deserializing size");

        let mut buffer = vec![0; (size + 8) as usize];

        let n = udp_socket
            .recv(&mut buffer)
            .expect("Error receiving packet");

        let buffer = &buffer[8..n];

        Ok(RtpPacket::decode(buffer))
    }

    fn send_rtsp_packet(&mut self, packet: RtspRequest) -> std::io::Result<()> {
        return self
            .server_connection
            .as_mut()
            .unwrap()
            .server_socket
            .write_all(&bincode::serialize(&packet).expect("Error serializing packet"));
    }

    fn receive_rtsp_packet(&mut self) -> std::io::Result<RtspResponse> {
        let mut buffer = [0; 1024];

        let n = self
            .server_connection
            .as_mut()
            .unwrap()
            .server_socket
            .read(&mut buffer)?;

        Ok(bincode::deserialize(&buffer[..n]).expect("Error deserializing packet"))
    }
}
