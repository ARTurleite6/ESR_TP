use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RtpPacket {
    version: u8,
    padding: u8,
    extension: u8,
    cc: u8,
    marker: u8,
    ssrc: u32,
    sequence_number: u16,
    timestamp: u32,
    payload_type: u8,
    payload: Vec<u8>,
    header: Vec<u8>,
}

impl RtpPacket {
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn transmit_data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.header.len() + self.payload.len());
        data.extend(&self.header);
        data.extend(&self.payload);
        dbg!(self.payload.len());
        data
    }

    pub fn decode(data: &[u8]) -> Self {
        let header = &data[0..12];

        let payload_type = header[1] & 127;
        let sequence_number = header[3] as u32 + 256 * header[2] as u32;
        let time_stamp = header[7] as u32
            + 256 * header[6] as u32
            + 65536 * header[5] as u32
            + 16777216 * header[4] as u32;

        RtpPacketBuilder::new(&data[12..], payload_type)
            .sequence_number(sequence_number as u16)
            .timestamp(time_stamp)
            .build()
    }
}

impl From<RtpPacketBuilder> for RtpPacket {
    fn from(value: RtpPacketBuilder) -> Self {
        let sequence_number = value.sequence_number.unwrap_or(0);
        let timestamp = value.timestamp.unwrap_or(0);
        let payload_type = value.payload_type;
        let ssrc = value.ssrc;

        let mut header = value.header;

        header[0] = value.version << 6 | value.padding << 5 | value.extension << 4 | value.cc;
        header[1] = value.marker << 7 | value.payload_type;
        header[2] = (sequence_number >> 8) as u8;
        header[3] = (sequence_number & 0xFF) as u8;
        header[4] = (timestamp >> 24) as u8;
        header[5] = (timestamp >> 16) as u8;
        header[6] = (timestamp >> 8) as u8;
        header[7] = (timestamp & 0xFF) as u8;
        header[8] = (ssrc >> 24) as u8;
        header[9] = (ssrc >> 16) as u8;
        header[10] = (ssrc >> 8) as u8;
        header[11] = (ssrc & 0xFF) as u8;

        Self {
            version: value.version,
            padding: value.padding,
            extension: value.extension,
            cc: value.cc,
            marker: value.marker,
            ssrc: value.ssrc,
            payload_type,
            payload: value.payload,
            sequence_number,
            timestamp,
            header,
        }
    }
}

#[derive(Debug, Default)]
pub struct RtpPacketBuilder {
    version: u8,
    padding: u8,
    extension: u8,
    cc: u8,
    marker: u8,
    ssrc: u32,
    sequence_number: Option<u16>,
    timestamp: Option<u32>,
    payload_type: u8,
    payload: Vec<u8>,
    header: Vec<u8>,
}

impl RtpPacketBuilder {
    pub fn new(data: &[u8], payload_type: u8) -> Self {
        let version = 2;
        let padding = 0;
        let extension = 0;
        let cc = 0;
        let marker = 0;
        let ssrc = 0;
        let payload = data.to_vec();

        Self {
            version,
            padding,
            payload_type,
            extension,
            cc,
            marker,
            ssrc,
            payload,
            header: vec![0; 12],
            ..Default::default()
        }
    }

    pub fn sequence_number(mut self, frame_nb: u16) -> Self {
        self.sequence_number = Some(frame_nb);
        self
    }

    pub fn timestamp(mut self, time: u32) -> Self {
        self.timestamp = Some(time);
        self
    }

    pub fn build(self) -> RtpPacket {
        RtpPacket::from(self)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_serialize() {
        use super::*;

        let packet = RtpPacketBuilder::new(&[0; 100], 0).build();

        let data = packet.transmit_data();

        let packet2 = RtpPacket::decode(&data);

        assert_eq!(packet2.payload(), &[0; 100])
    }
}
