use std::{
    net::{IpAddr, UdpSocket},
    sync::Mutex,
};

use crate::video::video_stream::VideoStream;

#[derive(Debug)]
pub struct VideoStreamInfo {
    video_stream: Mutex<VideoStream>,
    clients: Mutex<Vec<(IpAddr, u16)>>,
}

impl VideoStreamInfo {
    pub fn new(video_stream: VideoStream, clients: Vec<(IpAddr, u16)>) -> Self {
        Self {
            video_stream: Mutex::new(video_stream),
            clients: Mutex::new(clients),
        }
    }

    pub fn send_data(&self, rtp_socket: &UdpSocket) -> std::io::Result<()> {
        let mut video_lock = self.video_stream.lock().unwrap();
        let packet = video_lock.receive_next_packet()?;

        for client in self.clients.lock().unwrap().iter() {
            dbg!(client);
            let n = rtp_socket.send_to(&packet, client).unwrap();
            dbg!(n);
        }

        return Ok(());
    }

    pub fn add_client(&self, client: (IpAddr, u16)) -> usize {
        let mut lock = self.clients.lock().unwrap();
        lock.push(client);
        return lock.len();
    }

    pub fn remove_client(&self, client: (IpAddr, u16)) -> usize {
        let mut lock = self.clients.lock().unwrap();
        lock.retain(|c| c != &client);
        return lock.len();
    }

    pub fn has_clients(&self) -> bool {
        return !self.clients.lock().unwrap().is_empty();
    }
}
