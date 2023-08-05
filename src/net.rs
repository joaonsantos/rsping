pub mod icmp;

use std::net::IpAddr;

#[derive(Debug, Clone)]
pub enum ParseErrors {
    HostLookupErr(String),
    HostLookupEmpty,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    target: String,
    err: ParseErrors,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.err {
            ParseErrors::HostLookupEmpty => {
                write!(f, "got no results for lookup \"{}\"", self.target)
            }
            ParseErrors::HostLookupErr(err) => write!(
                f,
                "unable to parse due to failed lookup of \"{}\": {}",
                self.target, err
            ),
        }
    }
}

pub fn parse(target: &str) -> Result<IpAddr, ParseError> {
    // Try to parse target as an address.
    let addr: IpAddr = match target.parse() {
        Ok(v) => v,
        Err(_) => {
            // An error was found parsing the input target, assume input is an hostname
            // and try to lookup the respective hostname.
            match dns_lookup::lookup_host(&target) {
                Ok(addrs) => match addrs.first() {
                    Some(addr) => addr.to_owned(),
                    None => {
                        return Err(ParseError {
                            target: target.to_string(),
                            err: ParseErrors::HostLookupEmpty,
                        })
                    }
                },
                Err(e) => {
                    return Err(ParseError {
                        target: target.to_string(),
                        err: ParseErrors::HostLookupErr(e.to_string()),
                    })
                }
            }
        }
    };
    Ok(addr)
}
