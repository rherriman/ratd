pub mod config;
pub mod threading;

use std::{
    net::{SocketAddr, UdpSocket},
    sync::Arc
};

use super::protocol::{
    Command, Datagram, LobbyList,
    parse::TryParse
};
use self::{
    config::{Config, Error},
    threading::ThreadPool
};

pub struct Server {
    lobby_list: Arc<LobbyList>,
    socket: Arc<UdpSocket>,
    thread_pool: ThreadPool,
    verbose_logging: bool,
}

impl Server {
    pub fn new(config: Config) -> Result<Server, Error> {
        let lobby_list = Arc::new(LobbyList::new());
        let address = SocketAddr::from(([0; 4], config.port));
        let socket = Arc::new(UdpSocket::bind(address).map_err(|_| Error::SocketBindFailure)?);
        let thread_pool = ThreadPool::new(config.workers);
        let verbose_logging = config.verbose;
        Ok(Server { lobby_list, socket, thread_pool, verbose_logging })
    }

    pub fn run(&self) {
        loop {
            let mut buffer = [0; 8192];
            let (size, src) = match self.socket.recv_from(&mut buffer) {
                Ok(headers) => headers,
                Err(_) => {
                    eprintln!("ERROR: \"Failed to receive datagram\"");
                    continue;
                },
            };
            let lobby_list = Arc::clone(&self.lobby_list);
            let socket = Arc::clone(&self.socket);
            let verbose = self.verbose_logging;
            self.thread_pool.execute(move || {
                let contents = &buffer[..size];
                if verbose {
                    println!("Size: {}", size);
                    println!("Source Address: {}", src);
                    println!("Bytes: {:?}", contents);
                }

                let result = Datagram::try_parse(contents);
                match result {
                    Ok(datagram) => match datagram.get_command() {
                        Command::Query => {
                            // Safe to unwrap query id. If it wasn't, parsing would have failed.
                            let query_id = datagram.get_query_id().unwrap();
                            for outgoing in lobby_list.search(None, query_id, 500) {
                                if socket.send_to(&outgoing, src).is_err() && verbose {
                                    eprintln!("ERROR: \"Failed to send response\"");
                                }
                            }
                        },
                        Command::Response => { /* Tracker sends these but shouldn't receive! */ },
                        Command::Hello => lobby_list.insert(&src, &datagram),
                        Command::Goodbye => lobby_list.remove(&src),
                    },
                    Err(e) => {
                        eprintln!("ERROR: \"{}\" on received bytes: {:?}", e, contents);
                    },
                }
            });
        }
    }
}
