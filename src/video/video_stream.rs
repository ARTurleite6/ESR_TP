use std::{
    fs::File,
    io::{Read, Seek},
    path::Path,
};

use crate::message::rtp::RtpPacketBuilder;

const PACKET_TYPE: u8 = 26;

#[derive(Debug)]
pub struct VideoStream {
    file: File,
    frame_num: u32,
    file_size: u64,
}

impl VideoStream {
    pub fn new<P: AsRef<Path>>(file_name: P) -> std::io::Result<Self> {
        let file = File::open(file_name)?;

        let metadata = file.metadata()?;
        let file_size = metadata.len();

        return Ok(Self {
            file,
            frame_num: 0,
            file_size,
        });
    }

    pub fn receive_next_packet(&mut self) -> std::io::Result<Vec<u8>> {
        let data = self.next_frame()?;
        let frame_number = self.frame_num();

        let packet = RtpPacketBuilder::new(&data, PACKET_TYPE)
            .sequence_number(frame_number as u16)
            .build();

        let encode = packet.transmit_data();

        let size = encode.len() as u64;

        let size_encoded = bincode::serialize(&size).expect("Error serializing size");

        let mut encoded = size_encoded;
        encoded.extend(encode);

        return Ok(encoded);
    }

    fn loop_file(&mut self) {
        let current_position = self.file.seek(std::io::SeekFrom::Current(0)).unwrap();
        if current_position == self.file_size {
            self.file.seek(std::io::SeekFrom::Start(0)).unwrap();
        }
    }

    pub fn next_frame(&mut self) -> std::io::Result<Vec<u8>> {
        self.loop_file();

        let mut buffer = [0; 5];
        self.file.read_exact(&mut buffer)?;

        let buffer = String::from_utf8_lossy(&buffer);

        let frame_length = buffer.parse::<usize>().unwrap();

        let mut buffer = vec![0; frame_length];

        self.file.read_exact(&mut buffer)?;

        self.frame_num += 1;

        return Ok(buffer);
    }

    pub fn frame_num(&self) -> u32 {
        return self.frame_num;
    }
}
