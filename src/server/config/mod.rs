use std::{
    fmt,
    num::NonZeroUsize
};

use clap::ArgMatches;

#[derive(Debug)]
pub enum ConfigError {
    InvalidPortNumber = 1,
    InvalidWorkerCount,
    SocketBindFailure,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::InvalidPortNumber =>
                write!(f, "Port must be a number between 0 and 65535"),
            ConfigError::InvalidWorkerCount =>
                write!(f, "Worker count must be a number greater than 0"),
            ConfigError::SocketBindFailure =>
                write!(f, "Couldn't bind to address"),
        }
    }
}

pub struct Config {
    pub port: u16,
    pub workers: NonZeroUsize,
}

impl Config {
    pub fn from_clap(args: ArgMatches) -> Result<Config, ConfigError> {
        let mut config = Config::default();

        if args.is_present("port") {
            config.port = value_t!(args, "port", u16).map_err(|_| ConfigError::InvalidPortNumber)?;
        }

        if args.is_present("workers") {
            config.workers = match value_t!(args, "workers", usize) {
                Ok(workers) => {
                    if workers == 0 {
                        return Err(ConfigError::InvalidWorkerCount);
                    }

                    NonZeroUsize::new(workers).unwrap()
                },
                Err(_) => return Err(ConfigError::InvalidWorkerCount),
            };
        }

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            port: 21541,
            workers: NonZeroUsize::new(4).unwrap(),
        }
    }
}
