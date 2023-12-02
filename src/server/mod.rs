use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

mod metrics_worker;
pub mod rp;
pub mod server_worker;
pub mod transmission_channel;

use crate::server::server_worker::streaming_worker::StreamingWorker;

use self::server_worker::streaming_worker::transmission_worker::TransmissionWorker;

#[derive(Debug, Default)]
pub struct Server {
    metrics_port: u16,
    streaming_port: u16,
    files_available: Vec<String>,
    video_workers: Mutex<HashMap<String, Arc<TransmissionWorker>>>,
}

impl Server {
    pub fn new(metrics_port: u16, streaming_port: u16, files_available: Vec<String>) -> Self {
        //verify that the files are available
        //for file in &files_available {
        //    if !std::path::Path::new(file).exists() {
        //        panic!("File {} does not exist", file);
        //    }
        //}

        Self {
            metrics_port,
            streaming_port,
            files_available,
            ..Default::default()
        }
    }

    pub fn run(&self) {
        std::thread::scope(|s| {
            let streaming_listener = std::net::TcpListener::bind(("0.0.0.0", self.streaming_port)).unwrap();
            println!("Streaming socket listening on port {}", self.streaming_port);

            let metrics_listener = std::net::TcpListener::bind(("0.0.0.0", self.metrics_port)).unwrap();
            println!("Metrics socket listening on port {}", self.metrics_port);

            let streaming_port = streaming_listener.local_addr().unwrap().port();

            let metrics_worker = metrics_worker::MetricsWorker::new(
                streaming_port,
                metrics_listener,
                self
                    .files_available
                    .clone()
            );

            s.spawn(move || {
                metrics_worker.run();
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
