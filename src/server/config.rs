use std::{
    fmt,
    num::NonZeroUsize
};

use clap::ArgMatches;

#[derive(Debug)]
pub enum Error {
    InvalidPortNumber = 1,
    InvalidWorkerCount,
    SocketBindFailure,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidPortNumber =>
                write!(f, "Port must be a number between 0 and 65535"),
            Error::InvalidWorkerCount =>
                write!(f, "Worker count must be a number greater than 0"),
            Error::SocketBindFailure =>
                write!(f, "Couldn't bind to address"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Config {
    pub port: u16,
    pub verbose: bool,
    pub workers: NonZeroUsize,
}

impl Config {
    pub fn from_clap(args: &ArgMatches) -> Result<Config, Error> {
        let mut config = Config::default();

        if args.is_present("port") {
            config.port = value_t!(args, "port", u16).map_err(|_| Error::InvalidPortNumber)?;
        }

        if args.is_present("verbose") {
            config.verbose = true;
        }

        if args.is_present("workers") {
            config.workers = match value_t!(args, "workers", usize) {
                Ok(workers) => {
                    if workers == 0 {
                        return Err(Error::InvalidWorkerCount);
                    }

                    NonZeroUsize::new(workers).unwrap()
                },
                Err(_) => return Err(Error::InvalidWorkerCount),
            };
        }

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            port: 21541,
            verbose: false,
            workers: NonZeroUsize::new(4).unwrap(),
        }
    }
}
