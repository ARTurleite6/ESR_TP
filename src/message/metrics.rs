use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MetricsRequest {
    video_file: String,
    latency: u32,
}

impl MetricsRequest {
    pub fn new(video_file: String) -> Self {
        Self {
            video_file,
            ..Default::default()
        }
    }

    pub fn video_file(&self) -> &str {
        &self.video_file
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MetricsResponse {
    video_found: bool,
    already_streaming: bool,
    nr_videos_available: usize,
    nr_videos_already_streaming: usize,
    streaming_port: u16,
}

impl MetricsResponse {
    pub fn new(
        video_found: bool,
        already_streaming: bool,
        nr_videos_available: usize,
        nr_videos_already_streaming: usize,
        streaming_port: u16,
    ) -> Self {
        Self {
            video_found,
            already_streaming,
            nr_videos_available,
            nr_videos_already_streaming,
            streaming_port,
        }
    }

    pub fn metric_calculation(&self) -> f32 {
        let nr_videos_available_ratio = self.nr_videos_available as f32 * 0.3;
        let nr_videos_already_streaming_ratio = self.nr_videos_already_streaming as f32 * 0.7;
        nr_videos_already_streaming_ratio
            + nr_videos_available_ratio
            + self.already_streaming as u32 as f32
    }

    pub fn video_found(&self) -> bool {
        self.video_found
    }

    pub fn already_streaming(&self) -> bool {
        self.already_streaming
    }

    pub fn nr_videos_available(&self) -> usize {
        self.nr_videos_available
    }

    pub fn nr_videos_already_streaming(&self) -> usize {
        self.nr_videos_already_streaming
    }

    pub fn set_video_found(&mut self, video_found: bool) {
        self.video_found = video_found;
    }

    pub fn streaming_port(&self) -> u16 {
        self.streaming_port
    }
}
