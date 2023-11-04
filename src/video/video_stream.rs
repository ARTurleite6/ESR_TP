use std::{fs::File, io::Read};

pub struct VideoStream {
    file_name: String,
    file: File,
    frame_num: u32,
}

impl VideoStream {
    pub fn new(file_name: String) -> Self {
        let file = File::open(&file_name).expect("Error opening file");

        Self {
            file_name,
            file,
            frame_num: 0,
        }
    }

    pub fn next_frame(&mut self) -> Vec<u8> {
        let mut buffer = [0; 5];
        self.file.read_exact(&mut buffer).unwrap();

        let buffer = String::from_utf8_lossy(&buffer);

        let frame_length = buffer.parse::<usize>().unwrap();

        let mut buffer = vec![0; frame_length];

        self.file.read_exact(&mut buffer).unwrap();

        self.frame_num += 1;

        return buffer;
    }

    pub fn frame_num(&self) -> u32 {
        return self.frame_num;
    }
}
