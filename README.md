## rsping

`ping` clone implemented in Rust.


## Usage

Pass the UDP payload as an `utf-8` string and the target to ping as arguments. The UDP payload can be empty.

The target can be either an address or an hostname.

Both IPv4 and IPv6 are supported, hostname resolves by default to IPv6 if available.
```sh
$ rsping --help
Usage
  ./target/debug/rsping target payload
```

## Running

To run this program you need root permissions or set the necessary capabilities like distro maintainers do for the official ping binary.

The program needs the following capabilities:
```
CAP_NET_RAW
              •  Use RAW and PACKET sockets;
              •  bind to any address for transparent proxying.
```

To set binary capabilities:
```sh
$ sudo setcap 'cap_net_raw=ep' rsping
```

## Example

Ping `google.com` with an empty payload.
```sh
$ rsping google.com "" 
PING google.com 64 data bytes
64 bytes from 2a00:1450:4003:802::200e: icmp_seq=1 hops=3 time=8.10 ms
64 bytes from 2a00:1450:4003:802::200e: icmp_seq=2 hops=3 time=7.99 ms
28 bytes from 2a00:1450:4003:802::200e: icmp_seq=3 hops=3 time=0.13 ms
64 bytes from 2a00:1450:4003:802::200e: icmp_seq=4 hops=3 time=0.12 ms
64 bytes from 2a00:1450:4003:802::200e: icmp_seq=5 hops=3 time=0.14 ms
64 bytes from 2a00:1450:4003:802::200e: icmp_seq=6 hops=3 time=0.11 ms
64 bytes from 2a00:1450:4003:802::200e: icmp_seq=7 hops=3 time=0.12 ms
32 bytes from 2a00:1450:4003:802::200e: icmp_seq=8 hops=3 time=0.11 ms
64 bytes from 2a00:1450:4003:802::200e: icmp_seq=9 hops=3 time=0.13 ms
```

Press Ctrl+C at any time to exit.
