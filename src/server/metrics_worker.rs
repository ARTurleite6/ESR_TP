use std::{
    collections::HashSet,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{mpsc::Receiver, RwLock},
};

use esr_server::server_worker::streaming_worker::Message;

use esr_core::message::metrics::{MetricsRequest, MetricsResponse};

use esr_video::video_stream::VideoStream;

#[derive(Debug)]
pub struct MetricsWorker {
    metrics_listener: TcpListener,
    streaming_port: u16,
    videos_available: Vec<String>,
    video_workers: RwLock<HashSet<String>>,
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
            video_workers: Default::default(),
        }
    }

    fn handle_client(&self, mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let mut buffer = [0; 1024];

            let n = stream.read(&mut buffer)?;

            let metrics_request: MetricsRequest = bincode::deserialize(&buffer[..n])?;

            let video_file = metrics_request.video_file();

            let video_found = VideoStream::file_exists(video_file);
            let lock_guard = self.video_workers.read().unwrap();
            let already_streaming = lock_guard.contains(video_file);
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

    pub fn run(&self, receiver: Receiver<Message<String>>) {
        std::thread::scope(|s| {
            s.spawn(move || {
                for msg in receiver {
                    match msg {
                        Message::Add(video) => self.video_workers.write().unwrap().insert(video),
                        Message::Remove(video) => {
                            self.video_workers.write().unwrap().remove(&video)
                        }
                    };
                }
            });

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
