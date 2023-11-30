use std::{
    net::{IpAddr, UdpSocket},
    sync::Mutex,
};

use crate::{message::rtp::RtpPacketBuilder, video::video_stream::VideoStream};

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
        let data = video_lock.next_frame()?;
        let frame_number = video_lock.frame_num();

        drop(video_lock);

        let packet = RtpPacketBuilder::new(&data, 26)
            .sequence_number(frame_number as u16)
            .build();

        let encode = packet.transmit_data();

        let size = encode.len() as u64;

        let size_encoded = bincode::serialize(&size).expect("Error serializing size");

        let mut encoded = size_encoded;
        encoded.extend(encode);

        for client in self.clients.lock().unwrap().iter() {
            dbg!(client);
            let n = rtp_socket.send_to(&encoded, client).unwrap();
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
