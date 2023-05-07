## rsping

`ping` clone implemented in Rust.


## Usage
```sh
$ rsping --help
Usage
  ./target/debug/rsping target
```

## Running

To run this program you need root permissions.

```sh
$ rsping 185.199.111.153 
[sudo] password for jsantos: 
PING 185.199.111.153 64 data bytes
64 bytes from 185.199.111.153: icmp_seq=1 ttl=64 time=12.11 ms
PING 185.199.111.153 64 data bytes
64 bytes from 185.199.111.153: icmp_seq=2 ttl=64 time=12.40 ms
PING 185.199.111.153 64 data bytes
64 bytes from 185.199.111.153: icmp_seq=3 ttl=64 time=9.21 ms
PING 185.199.111.153 64 data bytes
64 bytes from 185.199.111.153: icmp_seq=4 ttl=64 time=9.98 ms
PING 185.199.111.153 64 data bytes
64 bytes from 185.199.111.153: icmp_seq=5 ttl=64 time=9.33 ms
PING 185.199.111.153 64 data bytes
64 bytes from 185.199.111.153: icmp_seq=6 ttl=64 time=9.46 ms
^C
received SIGINT, exiting...
```