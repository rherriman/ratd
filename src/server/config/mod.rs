use std::num::NonZeroUsize;

use clap::ArgMatches;

use ::server::error::RatdError;

pub struct Config {
    pub port: u16,
    pub workers: NonZeroUsize,
}

impl Config {
    pub fn from_clap(args: ArgMatches) -> Result<Config, RatdError> {
        let mut config = Config::default();

        if args.is_present("port") {
            config.port = match value_t!(args, "port", u16) {
                Ok(port) => port,
                Err(_) => return Err(RatdError::InvalidPortNumber),
            }
        }

        if args.is_present("workers") {
            config.workers = match value_t!(args, "workers", usize) {
                Ok(workers) => {
                    if workers == 0 {
                        return Err(RatdError::InvalidWorkerCount);
                    }

                    NonZeroUsize::new(workers).unwrap()
                },
                Err(_) => return Err(RatdError::InvalidWorkerCount),
            }
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
