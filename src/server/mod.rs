use std::net::{SocketAddr, UdpSocket};

use self::config::Config;
use self::error::RatdError;
use self::threading::ThreadPool;

pub mod config;
pub mod error;
pub mod threading;

pub struct Server {}

impl Server {
    pub fn run(config: Config) -> Result<(), RatdError> {
        let address = SocketAddr::from(([127, 0, 0, 1], config.port));
        let socket = match UdpSocket::bind(address) {
            Ok(socket) => socket,
            Err(_) => return Err(RatdError::SocketBindFailure),
        };
        let thread_pool = ThreadPool::new(config.workers);
        let mut buffer = [0; 8192];

        loop {
            let (size, src) = socket.recv_from(&mut buffer)
                .expect("Didn't receive data");
            println!("Size: {}", size);
            println!("Source Address: {}", src);
        }

        Ok(())
    }
}
