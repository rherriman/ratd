use std::{fmt, num::NonZeroU8};

use clap::{value_t, ArgMatches};

#[derive(Debug)]
pub enum Error {
    InvalidPortNumber = 1,
    InvalidLobbyTimeout,
    InvalidWorkerCount,
    SocketBindFailure,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidPortNumber => write!(f, "Port must be a number between 0 and 65535"),
            Error::InvalidLobbyTimeout => {
                write!(f, "Lobby timeout must be a number between 0 and 255")
            }
            Error::InvalidWorkerCount => {
                write!(f, "Worker count must be a number between 0 and 255")
            }
            Error::SocketBindFailure => write!(f, "Couldn't bind to address"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Config {
    pub port: u16,
    pub timeout: NonZeroU8,
    pub verbose: bool,
    pub workers: NonZeroU8,
}

impl Config {
    pub fn from_clap(args: &ArgMatches) -> Result<Config, Error> {
        let mut config = Config::default();

        if args.is_present("port") {
            config.port = value_t!(args, "port", u16).map_err(|_| Error::InvalidPortNumber)?;
        }

        if args.is_present("timeout") {
            config.timeout = match value_t!(args, "timeout", u8) {
                Ok(expiration_threshold) => {
                    if expiration_threshold == 0 {
                        return Err(Error::InvalidLobbyTimeout);
                    }

                    NonZeroU8::new(expiration_threshold).unwrap()
                }
                Err(_) => return Err(Error::InvalidLobbyTimeout),
            };
        }

        if args.is_present("verbose") {
            config.verbose = true;
        }

        if args.is_present("workers") {
            config.workers = match value_t!(args, "workers", u8) {
                Ok(workers) => {
                    if workers == 0 {
                        return Err(Error::InvalidWorkerCount);
                    }

                    NonZeroU8::new(workers).unwrap()
                }
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
            timeout: NonZeroU8::new(5).unwrap(),
            verbose: false,
            workers: NonZeroU8::new(4).unwrap(),
        }
    }
}
