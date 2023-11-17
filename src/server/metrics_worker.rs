use std::{io::Read, net::TcpListener};

use crate::message::metrics::{MetricsRequest, MetricsResponse};

#[derive(Debug)]
pub struct MetricsWorker<'a> {
    metrics_listener: TcpListener,
    streaming_port: u16,
    videos_available: &'a [&'a str],
}

impl<'a> MetricsWorker<'a> {
    pub fn new(
        streaming_port: u16,
        metrics_listener: TcpListener,
        videos_available: &'a [&'a str],
    ) -> Self {
        Self {
            streaming_port,
            metrics_listener,
            videos_available,
        }
    }

    pub fn run(&self) {
        std::thread::scope(|s| {
            for stream in self.metrics_listener.incoming() {
                todo!();
                let mut stream = stream.unwrap();

                s.spawn(|| {
                    let mut buffer = [0; 1024];

                    stream.read(&mut buffer).unwrap();

                    let metrics_request: MetricsRequest = bincode::deserialize(&buffer).unwrap();

                    let video_file = metrics_request.video_file();

                    let video_found = self.videos_available.contains(&video_file);

                    let metrics_response =
                        MetricsResponse::new(video_found, false, self.videos_available.len(), 0);
                });
            }
        });
    }
}
