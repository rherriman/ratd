pub mod datagram;
pub mod parse;
pub mod serialize;

use std::{
    cmp,
    collections::HashMap,
    net::SocketAddr,
    sync::RwLock,
    time::Instant
};

use self::serialize::Serialize;

const PROTOCOL_VERSION: u16 = 6;
pub const MAX_PLAYERS: u8 = 6;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Query,
    Response,
    Hello,
    Goodbye,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum GameStatus {
    NotLoaded,
    Loaded,
    Active,
    Paused,
}

#[derive(Debug, Clone)]
pub struct CommandPayload(Command);
#[derive(Debug, Clone)]
pub struct GameStatusPayload(GameStatus);
#[derive(Debug, Clone)]
pub struct BigIntPayload(u32);
#[derive(Debug, Clone)]
pub struct IntPayload(u16);
#[derive(Debug, Clone)]
pub struct SmallIntPayload(u8);
#[derive(Debug, Clone)]
pub struct RawStringPayload(Vec<u8>);

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerId {
    id: u8,
}

impl PlayerId {
    /// Create a new PlayerId.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the id is not a value between 0 and 5.
    fn new(id: u8) -> PlayerId {
        if id >= MAX_PLAYERS {
            panic!("Invalid player ID passed to PlayerId constructor");
        }
        PlayerId { id }
    }
}

#[derive(Debug, Clone)]
pub struct IndexedSocketAddrPayload(PlayerId, SocketAddr);
#[derive(Debug, Clone)]
pub struct IndexedRawStringPayload(PlayerId, RawStringPayload);
#[derive(Debug, Clone)]
pub struct IndexedIntPayload(PlayerId, IntPayload);
#[derive(Debug, Clone)]
pub struct IndexedLocationPayload(PlayerId, IntPayload, IntPayload);

#[derive(Debug, Clone)]
pub enum TrackerTag {
    Command(CommandPayload),
    QueryID(BigIntPayload),
    QueryString(RawStringPayload),
    HostDomain(RawStringPayload),
    ResponseIndex(IntPayload),
    ResponseCount(IntPayload),
    StatusMessage(RawStringPayload),
    InfoMessage(RawStringPayload),
    Invitation(RawStringPayload),
    HasPassword,
    PlayerLimit(SmallIntPayload),
    GameStatus(GameStatusPayload),
    LevelDirectory(RawStringPayload),
    LevelName(RawStringPayload),
    ProtocolVersion(IntPayload),
    SoftwareVersion(RawStringPayload),

    // (Indexed) Player fields.
    PlayerIPPort(IndexedSocketAddrPayload),
    PlayerNick(IndexedRawStringPayload),
    PlayerLives(IndexedIntPayload),
    PlayerLocation(IndexedLocationPayload),
}

impl TrackerTag {
    pub const NULL_ID: u8 = 0;
}

#[derive(Debug)]
pub struct Datagram {
    protocol_version: u16,
    command: Command,
    query_id: Option<u32>,
    tags: Vec<TrackerTag>,
}

impl Datagram {
    pub fn new(command: Command) -> Datagram {
        let tags = Vec::new();
        Datagram { protocol_version: PROTOCOL_VERSION, command, query_id: None, tags }
    }

    pub fn add_tag(&mut self, tag: TrackerTag) {
        self.tags.push(tag);
    }

    pub fn get_command(&self) -> Command {
        self.command
    }

    pub fn get_protocol_version(&self) -> u16 {
        self.protocol_version
    }

    pub fn get_query_id(&self) -> Option<u32> {
        self.query_id
    }

    pub fn set_query_id(&mut self, query_id: Option<u32>) {
        self.query_id = query_id;
    }
}

pub struct Player {
    id: PlayerId,
    net_address: SocketAddr,
    nickname: Vec<u8>,
    lives: u16,
    location: (u16, u16),
}

pub struct Lobby {
    preserialized: Vec<u8>,
    pub modified: Instant,
}

impl Lobby {
    pub fn new(datagram: &Datagram) -> Lobby {
        if datagram.command != Command::Hello {
            panic!("Lobby instance can only be created from \"hello\" datagrams");
        }
        let modified = Instant::now();
        let mut response = Datagram::new(Command::Response);
        response.tags = datagram.tags.clone();
        Lobby { preserialized: response.serialize(), modified }
    }

    pub fn as_response(&self, query_id: u32, response_index: u16, response_count: u16) -> Vec<u8> {
        let mut outgoing = self.preserialized.clone();
        outgoing.reserve(14);
        outgoing.append(&mut TrackerTag::QueryID(BigIntPayload(query_id)).serialize());
        outgoing.append(&mut TrackerTag::ResponseIndex(IntPayload(response_index)).serialize());
        outgoing.append(&mut TrackerTag::ResponseCount(IntPayload(response_count)).serialize());
        outgoing
    }
}

#[derive(Default)]
pub struct LobbyList {
    list: RwLock<HashMap<SocketAddr, Lobby>>,
}

impl LobbyList {
    pub fn new() -> LobbyList {
        let list = RwLock::new(HashMap::new());
        LobbyList { list }
    }

    pub fn insert(&self, key: &SocketAddr, datagram: &Datagram) {
        self.list.write().unwrap().insert(*key, Lobby::new(datagram));
    }

    pub fn remove(&self, key: &SocketAddr) {
        self.list.write().unwrap().remove(key);
    }

    pub fn search(&self, term: Option<&str>, query_id: u32, limit: u16) -> Vec<Vec<u8>> {
        let list = self.list.read().unwrap();
        let size = cmp::min(list.len(), usize::from(limit));
        let response_count = size as u16;
        let mut responses = Vec::with_capacity(size);

        // TODO: ACTUALLY FILTER, ATTACH INFO/STATUS MESSAGES

        if size == 0 {
            let mut datagram = Datagram::new(Command::Response);
            datagram.set_query_id(Some(query_id));
            datagram.add_tag(TrackerTag::ResponseCount(IntPayload(response_count)));
            responses.push(datagram.serialize());
        } else {
            let filtered_list = match term {
                Some(term) => list.iter().take(size),
                None => list.iter().take(size),
            };
            for (idx, (_, lobby)) in filtered_list.enumerate() {
                let response_index = idx as u16;
                responses.push(lobby.as_response(query_id, response_index, response_count));
            }
        }

        responses
    }
}

#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, Ipv4Addr},
        time::Duration
    };

    use super::*;

    fn build_hello() -> Datagram {
        let mut datagram = Datagram::new(Command::Hello);
        datagram.add_tag(TrackerTag::SoftwareVersion(RawStringPayload(vec![49, 46, 48, 46, 50])));
        datagram.add_tag(TrackerTag::PlayerLimit(SmallIntPayload(6)));
        datagram.add_tag(TrackerTag::Invitation(RawStringPayload(vec![73, 110, 118, 105, 116, 97, 116, 105, 111, 110, 32, 77, 101, 115, 115, 97, 103, 101])));
        datagram.add_tag(TrackerTag::HasPassword);
        datagram.add_tag(TrackerTag::PlayerIPPort(IndexedSocketAddrPayload(
            PlayerId::new(0),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 2, 15)), 19567)
        )));
        datagram.add_tag(TrackerTag::LevelDirectory(RawStringPayload(vec![65, 65, 32, 78, 111, 114, 109, 97, 108])));
        datagram.add_tag(TrackerTag::LevelName(RawStringPayload(vec![67, 111, 114, 111, 109, 111, 114, 97, 110])));
        datagram.add_tag(TrackerTag::GameStatus(GameStatusPayload(GameStatus::Active)));
        datagram.add_tag(TrackerTag::PlayerNick(IndexedRawStringPayload(
            PlayerId::new(0),
            RawStringPayload(vec![115, 105, 108, 118, 101, 114, 102, 111, 120])
        )));
        datagram.add_tag(TrackerTag::PlayerLocation(IndexedLocationPayload(
            PlayerId::new(0),
            IntPayload(7_233),
            IntPayload(46_424)
        )));
        datagram.add_tag(TrackerTag::PlayerLives(IndexedIntPayload(PlayerId::new(0), IntPayload(3))));
        datagram
    }

    #[test]
    fn valid_playerids() {
        for i in 0..MAX_PLAYERS {
            let _ = PlayerId::new(i);
        }
    }

    #[test]
    #[should_panic]
    fn invalid_playerid() {
        let _ = PlayerId::new(MAX_PLAYERS);
    }

    #[test]
    fn new_datagram() {
        let command = Command::Hello;
        let datagram = Datagram::new(command);
        assert_eq!(PROTOCOL_VERSION, datagram.get_protocol_version());
        assert_eq!(Command::Hello, datagram.get_command());
        assert_eq!(0, datagram.tags.len());
    }

    #[test]
    fn datagram_add_tag() {
        let command = Command::Hello;
        let mut datagram = Datagram::new(command);
        datagram.add_tag(TrackerTag::HasPassword);
        assert_eq!(PROTOCOL_VERSION, datagram.protocol_version);
        assert_eq!(Command::Hello, datagram.command);
        assert_eq!(1, datagram.tags.len());
    }

    #[test]
    fn datagram_query_id_getter_and_setter() {
        let command = Command::Response;
        let mut datagram = Datagram::new(command);
        assert_eq!(None, datagram.get_query_id());
        datagram.set_query_id(Some(3225));
        assert_eq!(Some(3225), datagram.get_query_id());
    }

    #[test]
    fn new_lobby() {
        let datagram = build_hello();
        let lobby = Lobby::new(&datagram);
        assert!(lobby.modified.elapsed() < Duration::from_secs(1));
    }

    #[test]
    #[should_panic]
    fn fail_new_lobby() {
        let mut datagram = build_hello();
        datagram.command = Command::Goodbye;
        let _ = Lobby::new(&datagram);
    }

    #[test]
    fn new_lobbylist() {
        let lobby_list = LobbyList::new();
        assert_eq!(0, lobby_list.list.read().unwrap().len());
    }

    #[test]
    fn lobbylist_insert() {
        let lobby_list = LobbyList::new();
        assert_eq!(0, lobby_list.list.read().unwrap().len());

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 2, 15)), 19567);
        let datagram = build_hello();

        lobby_list.insert(&addr, &datagram);
        assert_eq!(1, lobby_list.list.read().unwrap().len());
    }

    #[test]
    fn lobbylist_remove() {
        let lobby_list = LobbyList::new();
        assert_eq!(0, lobby_list.list.read().unwrap().len());

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 2, 15)), 19567);
        let datagram = build_hello();

        lobby_list.insert(&addr, &datagram);
        assert_eq!(1, lobby_list.list.read().unwrap().len());

        lobby_list.remove(&addr);
        assert_eq!(0, lobby_list.list.read().unwrap().len());
    }
}
