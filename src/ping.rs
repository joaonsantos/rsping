pub mod errors;

use socket2::{Domain, Protocol, SockAddr, Socket, Type};

use std::net::{IpAddr, SocketAddrV6};
use std::{mem::MaybeUninit, net::SocketAddrV4};

use crate::net::icmp;
use crate::ping::errors::*;

pub struct PingSendResult {
    pub payload_bytes: u64,
    pub seq: u16,
    pub ttl: String,
}

pub struct PingRecvResult {
    pub reply_bytes: u64,
}

pub struct Pinger {
    pub payload: String,
    pub socket: Option<Socket>,
    pub ttl: u32,
    pub seq: u16,
}

impl Pinger {
    pub fn new() -> Self {
        let ttl = icmp::TTL;
        let seq = 1;
        Self {
            payload: String::from(""),
            socket: None,
            ttl,
            seq,
        }
    }

    pub fn send(&mut self, addr: &IpAddr, payload: &str) -> Result<PingSendResult, PingSendError> {
        let packet = self.prepare_send(addr, payload)?;

        let mut buf: [u8; icmp::PACKET_SIZE] = [0; icmp::PACKET_SIZE];
        packet.encode(&mut buf);

        let payload_bytes: usize;
        if packet.is_ipv6() {
            let sock_addr: SocketAddrV6 = format!("[{}]:{}", addr, icmp::ECHO_REQUEST_PORT)
                .parse()
                .unwrap();
            payload_bytes = self
                .socket
                .as_ref()
                .ok_or(PingSendError {
                    target: addr.to_string(),
                    err: PingErrors::LookupErr,
                })?
                .send_to(&buf[..], &SockAddr::from(sock_addr))
                .map_err(|err| PingSendError {
                    target: sock_addr.to_string(),
                    err: PingErrors::PingErr(err.to_string()),
                })?;
        } else {
            let sock_addr: SocketAddrV4 = format!("{}:{}", addr, icmp::ECHO_REQUEST_PORT)
                .parse()
                .unwrap();
            payload_bytes = self
                .socket
                .as_ref()
                .ok_or(PingSendError {
                    target: addr.to_string(),
                    err: PingErrors::LookupErr,
                })?
                .send_to(&buf[..], &SockAddr::from(sock_addr))
                .map_err(|err| PingSendError {
                    target: sock_addr.to_string(),
                    err: PingErrors::PingErr(err.to_string()),
                })?;
        }

        let ttl_string = if packet.is_ipv6() {
            format!("hops={}", icmp::HOPS)
        } else {
            format!("ttl={}", icmp::TTL)
        };

        self.seq += 1;
        Ok(PingSendResult {
            payload_bytes: payload_bytes as u64,
            seq: self.seq - 1,
            ttl: ttl_string,
        })
    }

    pub fn recv(&self) -> Result<PingRecvResult, PingRecvErrs> {
        let mut reply_buf = [MaybeUninit::uninit(); icmp::PACKET_SIZE];
        let reply_bytes = self
            .socket
            .as_ref()
            .ok_or(PingRecvErrs::RecvErr("".to_string()))?
            .recv(reply_buf.as_mut_slice())
            .map_err(|err| PingRecvErrs::RecvErr(err.to_string()))?;

        Ok(PingRecvResult {
            reply_bytes: reply_bytes as u64,
        })
    }

    fn prepare_send(
        &mut self,
        addr: &IpAddr,
        payload: &str,
    ) -> Result<icmp::EchoRequestPacket, PingSendError> {
        // Get or lazily initialize socket.
        if addr.is_ipv6() {
            let sock = match &self.socket {
                Some(sock) => sock,
                None => self.socket.get_or_insert(
                    Socket::new(Domain::IPV6, Type::RAW, Some(Protocol::ICMPV6)).map_err(
                        |err| PingSendError {
                            target: addr.to_string(),
                            err: PingErrors::PingErr(err.to_string()),
                        },
                    )?,
                ),
            };
            sock.set_unicast_hops_v6(icmp::HOPS as u32)
                .unwrap_or_else(|_| ());
        } else {
            let sock = match &self.socket {
                Some(sock) => sock,
                None => self.socket.get_or_insert(
                    Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4)).map_err(
                        |err| PingSendError {
                            target: addr.to_string(),
                            err: PingErrors::PingErr(err.to_string()),
                        },
                    )?,
                ),
            };
            sock.set_ttl(self.ttl).unwrap_or_else(|_| ());
        };

        // Initialize request packet.
        let packet = if addr.is_ipv6() {
            icmp::EchoRequestPacket::V6(icmp::Echo6RequestPacket::new(String::from(payload)))
        } else {
            icmp::EchoRequestPacket::V4(icmp::Echo4RequestPacket::new(String::from(payload)))
        };

        Ok(packet)
    }
}
