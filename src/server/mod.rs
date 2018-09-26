pub mod config;
pub mod threading;

use std::net::{SocketAddr, UdpSocket};

use super::protocol::{Command, Datagram, LobbyList};
use self::{
    config::{Config, ConfigError},
    threading::ThreadPool
};

pub struct Server {
    lobby_list: LobbyList,
    socket: UdpSocket,
    thread_pool: ThreadPool,
}

impl Server {
    pub fn new(config: &Config) -> Result<Server, ConfigError> {
        let lobby_list = LobbyList::new();
        let address = SocketAddr::from(([0; 4], config.port));
        let socket = UdpSocket::bind(address).map_err(|_| ConfigError::SocketBindFailure)?;
        let thread_pool = ThreadPool::new(config.workers);
        Ok(Server { lobby_list, socket, thread_pool })
    }

    pub fn handle(&self, datagram: &Datagram, from_address: &SocketAddr) {
        match datagram.get_command() {
            Command::Query => {

            },
            Command::Response => {}, // Ignore. Tracker sends these out but shouldn't receive them.
            Command::Hello => {
                self.lobby_list.insert(from_address, datagram);
            },
            Command::Goodbye => {
                self.lobby_list.remove(from_address);
            },
        }
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
