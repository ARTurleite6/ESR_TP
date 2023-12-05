use core::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::o_node::neighbour::Neighbour;

#[derive(Error, Debug, Clone)]
pub enum RtpParsingError {
    #[error("Invalid format")]
    InvalidFormat,
    #[error("Invalid request type: {0}")]
    InvalidRequestType(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum RequestType {
    #[default]
    Setup,
    Play,
    Pause,
    Teardown,
}

impl fmt::Display for RequestType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Setup => write!(f, "SETUP"),
            Self::Play => write!(f, "PLAY"),
            Self::Pause => write!(f, "PAUSE"),
            Self::Teardown => write!(f, "TEARDOWN"),
        }
    }
}

impl FromStr for RequestType {
    type Err = RtpParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SETUP" => Ok(Self::Setup),
            "PLAY" => Ok(Self::Play),
            "PAUSE" => Ok(Self::Pause),
            "TEARDOWN" => Ok(Self::Teardown),
            _ => Err(RtpParsingError::InvalidRequestType(s.to_string())),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum Status {
    Ok = 200,
    FileNotFound = 404,
    ConnectionError = 500,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ok => write!(f, "OK"),
            Self::FileNotFound => write!(f, "NOT FOUND"),
            Self::ConnectionError => write!(f, "CONNECTION ERROR"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RtspResponse {
    status: Status,
    sequence: u32,
    session: u32,
}

impl RtspResponse {
    pub fn new(status: Status, sequence: u32, session: u32) -> Self {
        Self {
            status,
            sequence,
            session,
        }
    }

    pub fn succeded(&self) -> bool {
        self.status == Status::Ok
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn session_id(&self) -> u32 {
        self.session
    }
}

impl fmt::Display for RtspResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RTSP/1.0 {} {}\nCSeq: {}\nSession: {}\n",
            self.status as u16, self.status, self.sequence, self.session
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RtspRequest {
    request_type: RequestType,
    file_name: String,
    seq_number: u32,
    port_rtp: u16,
    servers_to_contact: Vec<Neighbour>,
}

impl fmt::Display for RtspRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} RTSP/1.0\nCSeq: {}\nTransport: RTP/UDP; client_port= {}\n",
            self.request_type, self.file_name, self.seq_number, self.port_rtp
        )
    }
}

impl RtspRequest {
    pub fn new(
        request_type: RequestType,
        file_name: String,
        seq_number: u32,
        port_rtp: u16,
    ) -> Self {
        Self {
            request_type,
            file_name,
            seq_number,
            port_rtp,
            ..Default::default()
        }
    }

    pub fn new_with_servers(
        request_type: RequestType,
        file_name: String,
        seq_number: u32,
        port_rtp: u16,
        servers_to_contact: Vec<Neighbour>,
    ) -> Self {
        Self {
            request_type,
            file_name,
            seq_number,
            port_rtp,
            servers_to_contact,
        }
    }

    pub fn request_type(&self) -> &RequestType {
        &self.request_type
    }

    pub fn file_request(&self) -> &str {
        &self.file_name
    }

    pub fn seq_number(&self) -> u32 {
        self.seq_number
    }

    pub fn port_rtp(&self) -> u16 {
        self.port_rtp
    }
    
    pub fn next_server(&mut self) -> Option<Neighbour> {
        self.servers_to_contact.pop()
    }
    
    pub fn servers_to_connect(&self) -> &Vec<Neighbour> {
        &self.servers_to_contact
    }
}

impl FromStr for RtspRequest {
    type Err = RtpParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut request = s.split('\n');

        let mut line1 = request.next().unwrap().split(' ');

        let request_type = line1.next().unwrap();

        let filename = line1.next().unwrap();

        let sequence_number = request
            .next()
            .unwrap()
            .split(' ')
            .nth(1)
            .unwrap()
            .parse()
            .unwrap();

        let port_rtp = request
            .next()
            .unwrap()
            .split(' ')
            .nth(3)
            .unwrap()
            .parse()
            .unwrap();

        Ok(Self {
            request_type: request_type.parse().unwrap(),
            file_name: filename.to_string(),
            seq_number: sequence_number,
            port_rtp,
            ..Default::default()
        })
    }
}
