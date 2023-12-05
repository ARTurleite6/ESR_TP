use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

use crate::message::metrics::{MetricsRequest, MetricsResponse};

#[derive(Debug)]
pub struct MetricsWorker {
    metrics_listener: TcpListener,
    streaming_port: u16,
    videos_available: Vec<String>,
}

impl MetricsWorker {
    pub fn new(
        streaming_port: u16,
        metrics_listener: TcpListener,
        videos_available: Vec<String>,
    ) -> Self {
        Self {
            streaming_port,
            metrics_listener,
            videos_available,
        }
    }

    fn handle_client(&self, mut stream: TcpStream) {
        loop {
            let mut buffer = [0; 1024];

            let n = stream.read(&mut buffer).unwrap();
            dbg!(&stream);

            let metrics_request: MetricsRequest = bincode::deserialize(&buffer[..n]).unwrap();

            let video_file = metrics_request.video_file();

            let video_found = self
                .videos_available
                .iter()
                .any(|video| video == video_file);

            let metrics_response = MetricsResponse::new(
                video_found,
                false,
                self.videos_available.len(),
                0,
                self.streaming_port,
            );

            let metrics_response = bincode::serialize(&metrics_response).unwrap();
            dbg!(&metrics_response);

            let n = stream.write(&metrics_response).unwrap();
            dbg!(n);
        }
    }

    pub fn run(&self) {
        std::thread::scope(|s| {
            for stream in self.metrics_listener.incoming() {
                let stream = stream.unwrap();

                dbg!("New client connected to metrics socket");

                s.spawn(move || self.handle_client(stream));
            }
        });
    }
}
