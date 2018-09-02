pub mod config;
pub mod protocol;
pub mod threading;

use std::net::{SocketAddr, UdpSocket};

use self::{
    config::{Config, ConfigError},
    protocol::Command,
    threading::ThreadPool
};

pub struct Server {
    socket: UdpSocket,
    thread_pool: ThreadPool,
}

impl Server {
    pub fn new(config: Config) -> Result<Server, ConfigError> {
        let address = SocketAddr::from(([0; 4], config.port));
        let socket = UdpSocket::bind(address).map_err(|_| ConfigError::SocketBindFailure)?;
        let thread_pool = ThreadPool::new(config.workers);
        Ok(Server { socket, thread_pool })
    }

    pub fn run(&self) {
        loop {
            let mut buffer = [0; 8192];
            let (size, src) = match self.socket.recv_from(&mut buffer) {
                Ok(headers) => headers,
                Err(_) => {
                    eprintln!("Failed to receive datagram");
                    continue;
                },
            };
            self.thread_pool.execute(move || {
                println!("Size: {}", size);
                println!("Source Address: {}", src);
                println!("Bytes: {:?}", &buffer[..size]);
            });
        }
    }
}
