use std::time::{Duration, Instant};

use crossbeam::{
    channel::{bounded, tick, Receiver},
    select,
};
use rsping::{
    net,
    ping::{PingRecvResult, PingSendResult, Pinger},
};

fn print_usage(cmd: String) {
    eprint!("Usage\n  {} payload target\n", cmd)
}

fn setup_sigint_handler() -> Receiver<()> {
    let (sender, receiver) = bounded(1);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })
    .expect("unable to setup SIGINT handler");

    receiver
}

fn main() {
    let mut args = std::env::args();
    let cmd = args.next().unwrap();
    if args.len() != 2 {
        print_usage(cmd);
        return;
    }

    let payload = args.next().unwrap_or("".to_string());
    if payload == "--help" {
        print_usage(cmd);
        return;
    }

    let target = args.next().unwrap_or("".to_string());
    if target == "" || target == "--help" {
        print_usage(cmd);
        return;
    }

    let addr = match net::parse(&target) {
        Ok(t) => t,
        Err(err) => {
            println!("Failed to parse target: {}\n", err);
            print_usage(cmd);
            return;
        }
    };
    let mut pinger = Pinger::new();

    let ticks = tick(Duration::from_secs(2));
    let ctrl_c_events = setup_sigint_handler();

    loop {
        select! {
            recv(ticks) -> _ => {
                let tstart = Instant::now();
                let ping_send_resp: PingSendResult;
                let ping_recv_resp: PingRecvResult;

                match pinger.send(&addr, &payload) {
                    Ok(r) => {ping_send_resp = r;},
                    Err(e) => {
                        eprintln!("{cmd}: {}", e);
                        continue;
                    }
                }

                // This message should only be printed after one send, which must match seq
                // number equal to 2.
                if pinger.seq == 2 {
                    println!("PING {} {} data bytes", target, ping_send_resp.payload_bytes);
                }

                match pinger.recv() {
                    Ok(r) => {ping_recv_resp = r;},
                    Err(e) => {
                        eprintln!("{cmd}: {}", e);
                        continue;
                    }
                }

                let rtt = tstart.elapsed().as_secs_f64() * 1000.0;
                println!(
                    "{} bytes from {}: icmp_seq={} {} time={:.2} ms",
                    ping_recv_resp.reply_bytes,
                    addr,
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

    if let Err(e) = pinger.send(&addr, &payload) {
        eprintln!("{cmd}: {}", e);
    }
}
