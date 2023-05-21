pub enum IcmpProto {
    V4,
    V6,
}

#[derive(Debug)]
pub struct PacketHeader {
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
pub struct Packet {
    pub header: PacketHeader,
    msg: String,
}

impl Packet {
    pub fn new(header: PacketHeader, msg: String) -> Self {
        Packet { header, msg }
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
