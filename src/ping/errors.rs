use std::{error, fmt};

#[derive(Debug, Clone)]
pub enum PingRecvErrs {
    RecvErr(String),
}

impl fmt::Display for PingRecvErrs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PingRecvErrs::RecvErr(err) => write!(f, "unable to recv: {}", err),
        }
    }
}

impl error::Error for PingRecvErrs {}

#[derive(Debug, Clone)]
pub enum PingErrors {
    LookupErr,
    PingErr(String),
}

#[derive(Debug, Clone)]
pub struct PingSendError {
    pub target: String,
    pub err: PingErrors,
}

impl fmt::Display for PingSendError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.err {
            PingErrors::LookupErr => write!(f, "unable to lookup \"{}\"", self.target),
            PingErrors::PingErr(err) => write!(f, "unable to ping \"{}\": {}", self.target, err),
        }
    }
}

impl error::Error for PingSendError {}
