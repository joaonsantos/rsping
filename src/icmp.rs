#[derive(Debug)]
pub struct Header {
    pub typ: u8,
    pub code: u8,
    pub checksum: u16,
    pub id: u16,
    pub seq: u16,
}

pub enum IcmpProto {
    V4,
    V6,
}

/// An ICMP echo packet implemented according to
/// https://en.wikipedia.org/wiki/Ping_(networking_utility)#ECHO-REQUEST.
///
#[derive(Debug)]
pub struct EchoRequestPacket {
    pub header: Header,
    msg: String,
}

impl EchoRequestPacket {
    const ICMPV4_REQUEST_TYPE: u8 = 8;
    const ICMPV6_REQUEST_TYPE: u8 = 128;

    pub fn new(proto: IcmpProto, msg: String) -> Self {
        let typ = match proto {
            IcmpProto::V4 => Self::ICMPV4_REQUEST_TYPE,
            IcmpProto::V6 => Self::ICMPV6_REQUEST_TYPE,
        };
        EchoRequestPacket {
            header: Header {
                typ,
                code: 0,
                checksum: 0,
                id: std::process::id() as u16,
                seq: 0,
            },
            msg,
        }
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

    pub fn set_seq(&mut self, seq: u16) {
        self.header.seq = seq;
    }
}

fn calculate_checksum(buf: &[u8]) -> u16 {
    let mut sum: u16 = 0;

    for word in buf.chunks(2) {
        let mut val = u16::from(word[0]) << 8;
        if word.len() > 1 {
            val = val | u16::from(word[1]);
        }
        sum += val;
    }
    let sum = !sum as u16;
    sum
}

#[cfg(test)]
mod tests {
    #[test]
    fn checksum_empty_packet() {
        let mut buf: [u8; 8] = [0; 8];
        let mut packet = super::EchoRequestPacket::new(super::IcmpProto::V4, "".to_string());
        packet.header.id = 0;
        packet.encode(&mut buf[..]);
        assert_eq!(u16::from_le_bytes([buf[2], buf[3]]), 65271);
    }

    #[test]
    fn checksum_msg_packet() {
        let mut buf: [u8; 12] = [0; 12];
        let mut packet = super::EchoRequestPacket::new(super::IcmpProto::V4, "TEST".to_string());
        packet.header.id = 0;
        packet.encode(&mut buf[..]);
        assert_eq!(u16::from_le_bytes([buf[2], buf[3]]), 25936);
    }
}
