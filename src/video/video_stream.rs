use std::{fs::File, io::Read, path::Path};

#[derive(Debug)]
pub struct VideoStream {
    file: File,
    frame_num: u32,
}

impl VideoStream {
    pub fn new<P: AsRef<Path>>(file_name: P) -> Self {
        let file = File::open(file_name).expect("Error opening file");

        Self { file, frame_num: 0 }
    }

    pub fn next_frame(&mut self) -> std::io::Result<Vec<u8>> {
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
