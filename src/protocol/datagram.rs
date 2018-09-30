use std::fmt;

use super::{MAX_PLAYERS, Command, GameStatus, Player, TrackerTag};

type PlayerList = [Option<Player>; MAX_PLAYERS as usize];

pub trait Datagram {}

pub trait UnidentifiedDatagram: Datagram {}

pub trait IdentifiedDatagram: Datagram {}

pub struct IncomingDatagram {
    protocol_version: u16,
    software_version: u16,
    command: Command,
    tags: Vec<TrackerTag>,
}

impl Datagram for IncomingDatagram {}
impl UnidentifiedDatagram for IncomingDatagram {}

pub struct QueryDatagram {
    protocol_version: u16,
    software_version: u16,
    query_id: u32,
    location: Option<(u16, u16)>,
    response_count: Option<u16>,
    query_string: Option<Vec<u8>>,
}

impl Datagram for QueryDatagram {}
impl IdentifiedDatagram for QueryDatagram {}

pub struct ResponseDatagram {
    protocol_version: u16,
    software_version: u16,
    query_id: u32,
    response_index: u16,
    response_count: u16,
    game_status: GameStatus,
    has_password: bool,
    player_limit: u8,
    players: PlayerList,
    host_domain: Option<Vec<u8>>,
    invitation: Option<Vec<u8>>,
    level_directory: Option<Vec<u8>>,
    level_name: Option<Vec<u8>>,
    status_message: Option<Vec<u8>>,
    info_message: Option<Vec<u8>>,
}

impl Datagram for ResponseDatagram {}
impl IdentifiedDatagram for ResponseDatagram {}

pub struct HelloDatagram {
    protocol_version: u16,
    software_version: u16,
    game_status: GameStatus,
    has_password: bool,
    player_limit: u8,
    players: PlayerList,
    invitation: Option<Vec<u8>>,
    level_directory: Option<Vec<u8>>,
    level_name: Option<Vec<u8>>,
}

impl Datagram for HelloDatagram {}
impl IdentifiedDatagram for HelloDatagram {}

pub struct GoodbyeDatagram {
    protocol_version: u16,
    software_version: u16,
    game_status: GameStatus,
    has_password: bool,
    player_limit: u8,
    players: PlayerList,
    invitation: Option<Vec<u8>>,
    level_directory: Option<Vec<u8>>,
    level_name: Option<Vec<u8>>,
}

impl Datagram for GoodbyeDatagram {}
impl IdentifiedDatagram for GoodbyeDatagram {}
