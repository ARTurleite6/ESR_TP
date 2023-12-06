use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

mod metrics_worker;
pub mod rp;
pub mod server_worker;
pub mod transmission_channel;

use crate::server::server_worker::streaming_worker::StreamingWorker;

use self::server_worker::streaming_worker::transmission_worker::TransmissionChannel;

#[derive(Debug, Default)]
pub struct Server {
    metrics_port: u16,
    streaming_port: u16,
    files_available: Vec<String>,
    video_workers: Mutex<HashMap<String, Arc<TransmissionChannel>>>,
}

impl Server {
    pub fn new(metrics_port: u16, streaming_port: u16) -> std::io::Result<Self> {
        let files_available = Self::get_files_available()?;

        dbg!(&files_available);

        Ok(Self {
            metrics_port,
            streaming_port,
            files_available,
            ..Default::default()
        })
    }

    fn get_files_available() -> std::io::Result<Vec<String>> {
        Ok(fs::read_dir(Path::new("videos"))?
            .map(|entry| {
                entry
                    .unwrap()
                    .file_name()
                    .into_string()
                    .unwrap()
                    .to_string()
            })
            .collect())
    }

    pub fn run(&self) {
        std::thread::scope(|s| {
            let streaming_listener =
                std::net::TcpListener::bind(("0.0.0.0", self.streaming_port)).unwrap();
            println!("Streaming socket listening on port {}", self.streaming_port);

            let metrics_listener =
                std::net::TcpListener::bind(("0.0.0.0", self.metrics_port)).unwrap();
            println!("Metrics socket listening on port {}", self.metrics_port);

            let streaming_port = streaming_listener.local_addr().unwrap().port();

            s.spawn(move || {
                metrics_worker::MetricsWorker::new(
                    streaming_port,
                    metrics_listener,
                    self.files_available.clone(),
                    &self.video_workers,
                )
                .run();
            });

            for stream in streaming_listener.incoming() {
                let stream = stream.unwrap();
                s.spawn(move || {
                    let mut worker = StreamingWorker::new(stream, &self.video_workers);
                    worker.run();
                });
            }
        });
    }
}
