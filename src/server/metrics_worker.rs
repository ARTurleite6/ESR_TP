use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

use crate::{
    message::metrics::{MetricsRequest, MetricsResponse},
    video::video_stream::VideoStream,
};

use super::server_worker::streaming_worker::transmission_worker::TransmissionChannel;

#[derive(Debug)]
pub struct MetricsWorker<'a> {
    metrics_listener: TcpListener,
    streaming_port: u16,
    videos_available: Vec<String>,
    video_workers: &'a Mutex<HashMap<String, Arc<TransmissionChannel>>>,
}

impl<'a> MetricsWorker<'a> {
    pub fn new(
        streaming_port: u16,
        metrics_listener: TcpListener,
        videos_available: Vec<String>,
        video_workers: &'a Mutex<HashMap<String, Arc<TransmissionChannel>>>,
    ) -> Self {
        Self {
            video_workers,
            streaming_port,
            metrics_listener,
            videos_available,
        }
    }

    fn handle_client(&self, mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let mut buffer = [0; 1024];

            let n = stream.read(&mut buffer)?;

            let metrics_request: MetricsRequest = bincode::deserialize(&buffer[..n])?;

            let video_file = metrics_request.video_file();

            let video_found = VideoStream::file_exists(video_file);
            let lock_guard = self.video_workers.lock().expect("Error aquiring the lock");
            let already_streaming = lock_guard.contains_key(video_file);
            let nr_videos_already_streaming = lock_guard.len();
            drop(lock_guard);

            let metrics_response = MetricsResponse::new(
                video_found,
                already_streaming,
                self.videos_available.len(),
                nr_videos_already_streaming,
                self.streaming_port,
            );

            let metrics_response = bincode::serialize(&metrics_response)?;

            let _ = stream.write(&metrics_response)?;
        }
    }

    pub fn run(&self) {
        std::thread::scope(|s| {
            for stream in self.metrics_listener.incoming() {
                let stream = stream.unwrap();

                s.spawn(move || {
                    if let Err(error) = self.handle_client(stream) {
                        println!("Error processing the request: {:?}", error);
                    }
                });
            }
        });
    }
}
