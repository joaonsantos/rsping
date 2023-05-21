use crate::icmp;

use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::error::{self};

use std::net::{IpAddr, SocketAddrV6};
use std::{mem::MaybeUninit, net::SocketAddrV4, time};

use crossbeam::channel::{bounded, select, tick, Receiver};
use std::fmt;
use std::time::Duration;

pub const ECHO_REQUEST4_TYPE: u8 = 8;
pub const ECHO_REQUEST4_CODE: u8 = 0;
pub const ECHO_REQUEST6_TYPE: u8 = 128;
pub const ECHO_REQUEST6_CODE: u8 = 0;
const ECHO_REQUEST_PORT: u8 = 0;

const PACKET_SIZE: usize = 64;
const TTL: u32 = 64;
const HOPS: u8 = 3;
const MSG: &str = "HELLO FROM RUST";

pub enum EchoRequestPacket {
    V4(Echo4RequestPacket),
    V6(Echo6RequestPacket),
}

pub struct Echo4RequestPacket {
    pub packet: icmp::Packet,
}

pub struct Echo6RequestPacket {
    pub packet: icmp::Packet,
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
        let header = icmp::PacketHeader {
            typ: ECHO_REQUEST4_TYPE,
            code: ECHO_REQUEST4_CODE,
            checksum: 0,
            id: std::process::id() as u16,
            seq: 0,
        };
        let packet = icmp::Packet::new(header, msg);
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
        let header = icmp::PacketHeader {
            typ: ECHO_REQUEST6_TYPE,
            code: ECHO_REQUEST6_CODE,
            checksum: 0,
            id: std::process::id() as u16,
            seq: 0,
        };
        let packet = icmp::Packet::new(header, msg);
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

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone)]
struct LookupErr;

impl fmt::Display for LookupErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "could not lookup target successfully")
    }
}

impl error::Error for LookupErr {}

pub struct Pinger {
    target: String,
    ttl: u32,
    seq: u16,
}

impl Pinger {
    pub fn new(target: String) -> Self {
        let ttl = TTL;
        let seq = 0;
        Self { target, ttl, seq }
    }

    pub fn ping(&mut self) -> Result<()> {
        // Try to parse target address.
        let addr: IpAddr = match self.target.parse() {
            Ok(v) => v,
            Err(_) => {
                // If some error is found parsing the address, assume that an hostname was used.
                //
                // Try to lookup the respective hostname.
                match dns_lookup::lookup_host(&self.target) {
                    Ok(addrs) => match addrs.first() {
                        Some(addr) => addr.to_owned(),
                        None => return Err(Box::new(LookupErr)),
                    },
                    Err(e) => return Err(Box::new(e)),
                }
            }
        };
        self.target = addr.to_string();

        let socket: Socket;
        let mut req: EchoRequestPacket;

        if addr.is_ipv6() {
            socket = Socket::new(Domain::IPV6, Type::RAW, Some(Protocol::ICMPV6))?;
            req = EchoRequestPacket::V6(Echo6RequestPacket::new(String::from(MSG)));
            socket.set_unicast_hops_v6(HOPS as u32)?;
        } else {
            socket = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))?;
            req = EchoRequestPacket::V4(Echo4RequestPacket::new(String::from(MSG)));
            socket.set_ttl(self.ttl)?;
        }

        let ctrl_c_events = Self::setup_sigint_handler();
        let ticks = tick(Duration::from_secs(2));

        loop {
            select! {
                recv(ticks) -> _ => {
                    self.seq += 1;
                    req.set_seq(self.seq);

                    ping_step(&socket, &mut req, &self.target)?;
                }
                recv(ctrl_c_events) -> _ => {
                    println!();
                    println!("received SIGINT, exiting...");
                    break;
                }
            }
        }

        Ok(())
    }

    fn setup_sigint_handler() -> Receiver<()> {
        let (sender, receiver) = bounded(1);
        ctrlc::set_handler(move || {
            let _ = sender.send(());
        })
        .expect("unable to setup SIGINT handler");

        receiver
    }
}

fn ping_step(socket: &socket2::Socket, req: &mut EchoRequestPacket, target: &str) -> Result<()> {
    let mut buf: [u8; PACKET_SIZE] = [0; PACKET_SIZE];
    req.encode(&mut buf);

    let tstart = time::Instant::now();

    let n: usize;
    if req.is_ipv6() {
        let addr: SocketAddrV6 = format!("[{}]:{}", target, ECHO_REQUEST_PORT)
            .parse()
            .unwrap();
        n = socket.send_to(&buf[..], &SockAddr::from(addr))?;
    } else {
        let addr: SocketAddrV4 = format!("{}:{}", target, ECHO_REQUEST_PORT).parse().unwrap();
        n = socket.send_to(&buf[..], &SockAddr::from(addr))?;
    }
    println!("PING {} {} data bytes", target, n);

    let mut reply_buf = [MaybeUninit::uninit(); PACKET_SIZE];
    let reply_bytes = socket.recv(reply_buf.as_mut_slice())?;
    let rtt = tstart.elapsed().as_secs_f64() * 1000.0;

    let ttl_string = if req.is_ipv6() {
        format!("hops={}", HOPS)
    } else {
        format!("ttl={}", TTL)
    };
    println!(
        "{} bytes from {}: icmp_seq={} {} time={:.2} ms",
        reply_bytes,
        target,
        req.get_seq(),
        ttl_string,
        rtt
    );
    Ok(())
}
