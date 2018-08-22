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
        let address = SocketAddr::from(([0; 4], config.port));
        let socket = match UdpSocket::bind(address) {
            Ok(socket) => socket,
            Err(_) => return Err(RatdError::SocketBindFailure),
        };
        let thread_pool = ThreadPool::new(config.workers);

        loop {
            let mut buffer = [0; 8192];
            let (size, src) = match socket.recv_from(&mut buffer) {
                Ok((size, src)) => (size, src),
                Err(_) => {
                    eprintln!("Failed to receive datagram");
                    continue;
                },
            };
            thread_pool.execute(move || {
                println!("Size: {}", size);
                println!("Source Address: {}", src);
                println!("Buffer: {:?}", &buffer[..128]);
            });
        }

        Ok(())
    }
}
