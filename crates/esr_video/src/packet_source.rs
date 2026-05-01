use std::net::UdpSocket;

pub trait PacketSource {
    fn receive_next_packet(&self) -> std::io::Result<Vec<u8>>;
}

impl PacketSource for UdpSocket {
    fn receive_next_packet(&self) -> std::io::Result<Vec<u8>> {
        let mut buffer_size = [0; 8];

        self.peek(&mut buffer_size)
            .expect("Error getting size of buffer");

        let size: u64 = bincode::deserialize(&buffer_size).expect("Error deserializing size");

        let mut buffer = vec![0; (size + 8) as usize];

        let n = self.recv(&mut buffer).expect("Error receiving packet");

        let buffer = &buffer[0..n];

        Ok(buffer.into())
    }
}
