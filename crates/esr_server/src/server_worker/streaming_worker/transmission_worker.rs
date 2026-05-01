use std::{
    net::{IpAddr, UdpSocket},
    sync::mpsc,
    time::Duration,
};

use crate::server_worker::streaming_worker::{Message, StreamingMessage};

use super::video_stream_info::VideoStreamInfo;

#[derive(Debug)]
pub struct TransmissionChannel {
    video_client_addrs: VideoStreamInfo,
}

pub fn run(
    rtp_socket: UdpSocket,
    video_client_addrs: VideoStreamInfo,
    receiver: mpsc::Receiver<StreamingMessage>,
) {
    loop {
        while let Ok(msg) = receiver.try_recv() {
            match msg {
                Message::Add(addr) => video_client_addrs.add_client(addr),
                Message::Remove(addr) => video_client_addrs.remove_client(addr),
            };
        }

        std::thread::sleep(Duration::from_secs_f64(0.05));

        if !video_client_addrs.has_clients() {
            println!("Worker stopped running: There are no more clients");
            break;
        }

        if video_client_addrs.send_data(&rtp_socket).is_err() {
            println!("Reached the end of the video");
            break;
        }
    }
}

impl TransmissionChannel {
    pub fn new(video_client_addrs: VideoStreamInfo) -> Self {
        Self { video_client_addrs }
    }

    pub fn add_client(&self, client: (IpAddr, u16)) -> usize {
        self.video_client_addrs.add_client(client)
    }

    pub fn remove_client(&self, client: (IpAddr, u16)) -> usize {
        self.video_client_addrs.remove_client(client)
    }

    pub fn has_clients(&self) -> bool {
        self.video_client_addrs.has_clients()
    }
}
