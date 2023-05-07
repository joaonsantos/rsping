use crate::icmp;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::{mem::MaybeUninit, net::SocketAddrV4, time};

use crossbeam::channel::{bounded, select, tick, Receiver};
use std::io;
use std::time::Duration;

pub struct Pinger {
    target: String,
    port: String,
    ttl: u32,
    seq: u16,
}

impl Pinger {
    const PACKET_SIZE: usize = 64;
    const TTL: u32 = 64;
    const MSG: &str = "HELLO FROM RUST";

    pub fn new(target: String, port: String) -> Self {
        let ttl = Self::TTL;
        let seq = 0;
        Self {
            target,
            port,
            ttl,
            seq,
        }
    }

    pub fn ping(&mut self) -> Result<(), io::Error> {
        let socket = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))?;
        socket.set_ttl(self.ttl)?;

        let msg = String::from(Self::MSG);
        let mut icmp_packet = icmp::EchoRequestPacket::new(icmp::IcmpProto::V4, msg);

        let ctrl_c_events = Self::setup_sigint_handler();
        let ticks = tick(Duration::from_secs(2));

        loop {
            select! {
                recv(ticks) -> _ => {
                    self.seq += 1;
                    icmp_packet.set_seq(self.seq);

                    self.ping_step(&socket, &icmp_packet)?;
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

    fn ping_step(
        &self,
        socket: &socket2::Socket,
        icmp_packet: &icmp::EchoRequestPacket,
    ) -> Result<(), io::Error> {
        let mut buf: [u8; Self::PACKET_SIZE] = [0; Self::PACKET_SIZE];
        icmp_packet.encode(&mut buf);

        let addr: SocketAddrV4 = format!("{}:{}", self.target, self.port).parse().unwrap();

        let tstart = time::Instant::now();
        let n = socket.send_to(&buf[..], &SockAddr::from(addr))?;
        println!("PING {} {} data bytes", self.target, n);

        let mut reply_buf = [MaybeUninit::uninit(); Self::PACKET_SIZE];
        let reply_bytes = socket.recv(reply_buf.as_mut_slice())?;
        let rtt = tstart.elapsed().as_secs_f64() * 1000.0;
        println!(
            "{} bytes from {}: icmp_seq={} ttl={} time={:.2} ms",
            reply_bytes, self.target, icmp_packet.header.seq, self.ttl, rtt
        );
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
