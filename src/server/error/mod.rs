use std::fmt;

pub enum RatdError {
    InvalidPortNumber = 1,
    InvalidWorkerCount,
    SocketBindFailure,
}

impl fmt::Display for RatdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RatdError::InvalidPortNumber =>
                write!(f, "Port must be a number between 0 and 65535"),
            RatdError::InvalidWorkerCount =>
                write!(f, "Worker count must be a number greater than 0"),
            RatdError::SocketBindFailure =>
                write!(f, "Couldn't bind to address"),
        }
    }
}
