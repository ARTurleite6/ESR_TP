use std::net::TcpStream;

pub struct ServerWorker {
    rtsp_socket: TcpStream,
}

impl ServerWorker {
    pub fn new(rtsp_socket: TcpStream) -> Self {
        Self {
            rtsp_socket,
        }
    }

    pub fn run(&self) {

    }
}
