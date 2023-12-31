use std::{
    net::{IpAddr, UdpSocket},
    sync::Arc,
    time::Duration,
};

use super::video_stream_info::VideoStreamInfo;

#[derive(Debug)]
pub struct TransmissionChannel {
    rtp_socket: Arc<UdpSocket>,
    video_client_addrs: Arc<VideoStreamInfo>,
}

impl TransmissionChannel {
    pub fn new(rtp_socket: Arc<UdpSocket>, video_client_addrs: Arc<VideoStreamInfo>) -> Self {
        Self {
            rtp_socket,
            video_client_addrs,
        }
    }

    pub fn run(&self) {
        loop {
            std::thread::sleep(Duration::from_secs_f64(0.05));

            if !self.video_client_addrs.has_clients() {
                println!("Worker stopped running: There are no more clients");
                break;
            }

            if self.video_client_addrs.send_data(&self.rtp_socket).is_err() {
                println!("Reached the end of the video");
                break;
            }
        }
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
