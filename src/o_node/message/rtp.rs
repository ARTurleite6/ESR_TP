#[derive(Debug)]
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

impl From<RtpPacketBuilder> for RtpPacket {
    fn from(value: RtpPacketBuilder) -> Self {
        let sequence_number = value.sequence_number.unwrap_or(0);
        let timestamp = value.timestamp.unwrap_or(0);
        let payload_type = value.payload_type;
        let ssrc = value.ssrc;

        let mut header = value.header;

        header[0] =
            (value.version << 6 | value.padding << 5 | value.extension << 4 | value.cc) as u8;
        header[1] = (value.marker << 7 | value.payload_type & 0x000000FF) as u8;
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
            header: vec![0, 12],
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
        return RtpPacket::from(self);
    }
}
