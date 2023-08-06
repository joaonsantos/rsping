use std::net::IpAddr;
use std::time::{Duration, Instant};

use crossbeam::{
    channel::{bounded, tick, Receiver},
    select,
};
use rsping::{
    net::{self, icmp},
    ping::{PingRecvResult, PingSendResult, Pinger, TimeoutOption},
};

fn print_usage(cmd: String) {
    eprint!("Usage\n  {} target payload\n", cmd)
}

fn setup_sigint_handler() -> Receiver<()> {
    let (sender, receiver) = bounded(1);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })
    .expect("unable to setup SIGINT handler");

    receiver
}

struct Args {
    cmd: String,
    addr: IpAddr,
    payload: String,
}

fn parse_args() -> Args {
    let mut args = std::env::args();
    let cmd = args.next().unwrap();
    if args.len() != 2 {
        print_usage(cmd);
        std::process::exit(2);
    }

    let target = args.next().unwrap_or("".to_string());
    if target == "" || target == "--help" {
        print_usage(cmd);
        std::process::exit(2);
    }

    let payload = args.next().unwrap_or("".to_string());
    if payload == "--help" {
        print_usage(cmd);
        std::process::exit(2);
    }

    let addr = match net::parse(&target) {
        Ok(t) => t,
        Err(err) => {
            println!("failed to parse target: {}\n", err);
            print_usage(cmd);
            std::process::exit(2);
        }
    };

    Args { cmd, addr, payload }
}

fn main() {
    let args = parse_args();
    let loop_timeout = 2u32;
    let socket_timeout = 2u32;

    // Create pinger.
    let timeout = if args.addr.is_ipv6() {
        TimeoutOption::HOPS(icmp::DEFAULT_HOPS)
    } else {
        TimeoutOption::TTL(icmp::DEFAULT_TTL)
    };

    let mut pinger = Pinger::new(timeout, socket_timeout);

    // Setup ping loop to ping every few seconds while watching for SIGINT.
    let ticks = tick(Duration::from_secs(loop_timeout.into()));
    let ctrl_c_events = setup_sigint_handler();

    loop {
        select! {
            recv(ticks) -> _ => {
                let tstart = Instant::now();
                let ping_send_resp: PingSendResult;
                let ping_recv_resp: PingRecvResult;

                match pinger.send(&args.addr, &args.payload) {
                    Ok(r) => {
                        ping_send_resp = r;
                    },
                    Err(e) => {
                        eprintln!("{}: {}",args.cmd, e);
                        continue;
                    }
                }

                // This message should only be printed after the first send, which must match seq
                // number equal to 1.
                if ping_send_resp.seq == 1 {
                    println!("PING {} {} data bytes", args.addr.to_string(), ping_send_resp.payload_bytes);
                }

                // Wait for any any valid response.
                match pinger.recv() {
                    Ok(r) => {
                        if r.reply_bytes == 0 {
                            continue;
                        }
                        ping_recv_resp = r;
                    },
                    Err(e) => {
                        eprintln!("{}: {}",args.cmd, e);
                        continue;
                    }
                }

                let rtt = tstart.elapsed().as_secs_f64() * 1000.0;
                println!(
                    "{} bytes from {}: icmp_seq={} {} time={:.2} ms",
                    ping_recv_resp.reply_bytes,
                    args.addr.to_string(),
                    ping_send_resp.seq,
                    ping_send_resp.ttl,
                    rtt
                );
            }
            recv(ctrl_c_events) -> _ => {
                println!();
                break;
            }
        }
    }
}
