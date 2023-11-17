use std::{collections::HashMap, sync::RwLock};

use self::server_worker::{StreamingWorker, TransmissionWorker};

mod metrics_worker;
pub mod server_worker;

#[derive(Debug, Default)]
pub struct Server {
    port: u16,
    files_available: Vec<String>,
    video_workers: HashMap<String, TransmissionWorker>,
}

impl Server {
    pub fn new(port: u16, files_available: Vec<String>) -> Self {
        //verify that the files are available
        for file in &files_available {
            if !std::path::Path::new(file).exists() {
                panic!("File {} does not exist", file);
            }
        }

        Self {
            port,
            files_available,
            ..Default::default()
        }
    }

    pub fn run(&self) {
        std::thread::scope(|s| {
            let streaming_listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            println!("Listening on port {}", self.port);

            let metrics_listener = std::net::TcpListener::bind(("127.0.0.1", self.port)).unwrap();

            let streaming_port = streaming_listener.local_addr().unwrap().port();

            let metrics_worker = metrics_worker::MetricsWorker::new(
                streaming_port,
                metrics_listener,
                &self
                    .files_available
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<&str>>(),
            );

            s.spawn(move || {
                metrics_worker.run();
            });

            for stream in streaming_listener.incoming() {
                let stream = stream.unwrap();
                s.spawn(move || {
                    let mut worker = StreamingWorker::new(stream);
                    worker.run();
                });
            }
        });
    }
}
