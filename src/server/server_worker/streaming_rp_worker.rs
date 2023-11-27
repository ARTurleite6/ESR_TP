use std::{sync::{Mutex, Arc}, net::TcpStream, collections::HashMap};

use crate::server::rp::TransmissionChannel;

pub struct StreamingWorker<'a> {
    stream: TcpStream,
    video_workers: &'a Mutex<HashMap<String, Arc<TransmissionChannel>>>,
}