pub const ECHO_REQUEST4_TYPE: u8 = 8;
pub const ECHO_REQUEST4_CODE: u8 = 0;
pub const ECHO_REQUEST6_TYPE: u8 = 128;
pub const ECHO_REQUEST6_CODE: u8 = 0;
pub const ECHO_REQUEST_PORT: u8 = 0;

pub const PACKET_SIZE: usize = 64;

pub const DEFAULT_TTL: u32 = 64;
pub const DEFAULT_HOPS: u32 = 3;

pub enum IcmpProto {
    V4,
    V6,
}

#[derive(Debug)]
pub struct EchoPacketHeader {
    pub typ: u8,
    pub code: u8,
    pub checksum: u16,
    pub id: u16,
    pub seq: u16,
}

/// An ICMP echo packet implemented according to
/// https://en.wikipedia.org/wiki/Ping_(networking_utility)#ECHO-REQUEST.
///
#[derive(Debug)]
pub struct EchoPacket {
    pub header: EchoPacketHeader,
    msg: String,
}

impl EchoPacket {
    pub fn new(header: EchoPacketHeader, msg: String) -> Self {
        EchoPacket { header, msg }
    }
    pub fn encode(&self, buf: &mut [u8]) {
        buf[0] = self.header.typ;
        buf[1] = self.header.code;
        buf[4] = (self.header.id >> 8) as u8;
        buf[5] = self.header.id as u8;
        buf[6] = (self.header.seq >> 8) as u8;
        buf[7] = self.header.seq as u8;

        if self.msg != "" {
            let msg = self.msg.as_bytes();
            let copy_upper_bound = 8 + msg.len();
            buf[8..copy_upper_bound].copy_from_slice(msg);
        }

        let checksum = calculate_checksum(&buf);
        buf[2] = (checksum >> 8) as u8;
        buf[3] = checksum as u8;
    }
}

/// Calculate checksum according to https://en.wikipedia.org/wiki/Internet_checksum.
fn calculate_checksum(buf: &[u8]) -> u16 {
    // Use 32 bits to account for carry bits.
    let mut sum: u32 = 0;

    for word in buf.chunks(2) {
        let mut val = u16::from(word[0]) << 8;
        if word.len() > 1 {
            val += u16::from(word[1]);
        }
        sum = sum.wrapping_add(u32::from(val));
    }

    // Sum carry bits with the 16 least significant bits.
    sum = (sum >> 16) + (sum & 0xffff);

    // Calculate the ones' complement using logical negation.
    let sum = !sum as u16;
    sum
}

pub enum EchoRequestPacket {
    V4(Echo4RequestPacket),
    V6(Echo6RequestPacket),
}

pub struct Echo4RequestPacket {
    pub packet: EchoPacket,
}

pub struct Echo6RequestPacket {
    pub packet: EchoPacket,
}

impl EchoRequestPacket {
    pub fn encode(&self, buf: &mut [u8]) {
        match self {
            EchoRequestPacket::V4(packet) => packet.encode(buf),
            EchoRequestPacket::V6(packet) => packet.encode(buf),
        }
    }

    pub fn get_seq(&mut self) -> u16 {
        match self {
            EchoRequestPacket::V4(packet) => packet.get_seq(),
            EchoRequestPacket::V6(packet) => packet.get_seq(),
        }
    }

    pub fn set_seq(&mut self, seq: u16) {
        match self {
            EchoRequestPacket::V4(packet) => packet.set_seq(seq),
            EchoRequestPacket::V6(packet) => packet.set_seq(seq),
        }
    }

    pub fn is_ipv6(&self) -> bool {
        return std::matches!(self, EchoRequestPacket::V6(_));
    }
}

impl Echo4RequestPacket {
    pub fn new(msg: String) -> Self {
        let header = EchoPacketHeader {
            typ: ECHO_REQUEST4_TYPE,
            code: ECHO_REQUEST4_CODE,
            checksum: 0,
            id: std::process::id() as u16,
            seq: 0,
        };
        let packet = EchoPacket::new(header, msg);
        Echo4RequestPacket { packet }
    }

    pub fn encode(&self, buf: &mut [u8]) {
        self.packet.encode(buf)
    }

    pub fn get_seq(&mut self) -> u16 {
        self.packet.header.seq
    }

    pub fn set_seq(&mut self, seq: u16) {
        self.packet.header.seq = seq;
    }
}

impl Echo6RequestPacket {
    pub fn new(msg: String) -> Self {
        let header = EchoPacketHeader {
            typ: ECHO_REQUEST6_TYPE,
            code: ECHO_REQUEST6_CODE,
            checksum: 0,
            id: std::process::id() as u16,
            seq: 0,
        };
        let packet = EchoPacket::new(header, msg);
        Echo6RequestPacket { packet }
    }

    pub fn encode(&self, buf: &mut [u8]) {
        self.packet.encode(buf)
    }

    pub fn get_seq(&mut self) -> u16 {
        self.packet.header.seq
    }

    pub fn set_seq(&mut self, seq: u16) {
        self.packet.header.seq = seq;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn checksum_empty_packet() {
        let mut buf: [u8; 8] = [0; 8];
        let mut req = super::Echo4RequestPacket::new("".to_string());
        req.packet.header.id = 0;
        req.packet.encode(&mut buf[..]);
        assert_eq!(u16::from_le_bytes([buf[2], buf[3]]), 65527);
    }
    #[test]
    fn checksum_empty_packet_v6() {
        let mut buf: [u8; 8] = [0; 8];
        let mut req = super::Echo6RequestPacket::new("".to_string());
        req.packet.header.id = 0;
        req.packet.encode(&mut buf[..]);
        assert_eq!(u16::from_le_bytes([buf[2], buf[3]]), 65407);
    }
    #[test]
    fn checksum_msg_packet() {
        let mut buf: [u8; 12] = [0; 12];
        let mut req = super::Echo4RequestPacket::new("TEST".to_string());
        req.packet.header.id = 0;
        req.packet.encode(&mut buf[..]);
        assert_eq!(u16::from_le_bytes([buf[2], buf[3]]), 26192);
    }
    #[test]
    fn checksum_msg_packet_v6() {
        let mut buf: [u8; 12] = [0; 12];
        let mut req = super::Echo6RequestPacket::new("TEST".to_string());
        req.packet.header.id = 0;
        req.packet.encode(&mut buf[..]);
        assert_eq!(u16::from_le_bytes([buf[2], buf[3]]), 26072);
    }
}
