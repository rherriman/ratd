pub mod parse;
pub mod serialize;

use std::{
    cmp,
    collections::HashMap,
    net::SocketAddr,
    sync::RwLock,
    time::{Duration, Instant},
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

#[derive(Debug, Clone, Copy, PartialEq)]
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
    QueryId(BigIntPayload),
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
    PlayerIpPort(IndexedSocketAddrPayload),
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
    host_address: Option<SocketAddr>,
    query_id: Option<u32>,
    tags: Vec<TrackerTag>,
}

impl Datagram {
    pub fn new(command: Command) -> Datagram {
        Datagram {
            protocol_version: PROTOCOL_VERSION,
            command,
            host_address: None,
            query_id: None,
            tags: Vec::new(),
        }
    }

    pub fn add_tag(&mut self, tag: TrackerTag) {
        if let TrackerTag::PlayerIpPort(IndexedSocketAddrPayload(id, addr)) = tag {
            if id == PlayerId::new(0) {
                self.host_address = Some(addr);
            }
        }
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

pub struct Lobby {
    preserialized: Vec<u8>,
    pub modified: Instant,
}

impl Lobby {
    /// Create a new `Lobby`.
    ///
    /// # Arguments
    ///
    /// * `real_addr` - The `SocketAddr` from which the host contacted us. Very important, because Avara "lies" when it self-reports the host's IP. (Routers were not yet commonplace in 1996.)
    /// * `datagram` - The received `Command::Hello` `Datagram` on which to base this `Lobby`.
    pub fn new(real_addr: &SocketAddr, datagram: &Datagram) -> Lobby {
        if datagram.command != Command::Hello {
            panic!("Lobby instance can only be created from \"hello\" datagrams");
        }
        let modified = Instant::now();
        let mut response = Datagram::new(Command::Response);
        response.host_address = datagram.host_address;
        response.tags = datagram.tags.clone();
        for tag in response.tags.iter_mut() {
            if let TrackerTag::PlayerIpPort(IndexedSocketAddrPayload(id, addr)) = tag {
                if *id == PlayerId::new(0) {
                    let new_addr = SocketAddr::new(real_addr.ip(), addr.port());
                    *tag = TrackerTag::PlayerIpPort(IndexedSocketAddrPayload(*id, new_addr));
                    response.host_address = Some(*real_addr);
                }
            }
        }
        Lobby {
            preserialized: response.serialize(),
            modified,
        }
    }

    /// Returns a serialized version of a `Command::Response` `Datagram` for this `Lobby`.
    ///
    /// # Arguments
    ///
    /// * `query_id` - The query ID to which we are responding. (This value is typically received from an incoming `Command::Query` `Datagram`.)
    /// * `response_index` - The index of this `Command::Response` `Datagram` in relation to the rest of the outgoing batch.
    /// * `response_count` - The total number of `Command::Response` `Datagram`s in the outgoing batch.
    pub fn as_serialized_response(
        &self,
        query_id: u32,
        response_index: u16,
        response_count: u16,
    ) -> Vec<u8> {
        let mut outgoing = self.preserialized.clone();
        outgoing.reserve(14);
        outgoing.append(&mut TrackerTag::QueryId(BigIntPayload(query_id)).serialize());
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
    /// Create a new `LobbyList`.
    pub fn new() -> LobbyList {
        let list = RwLock::new(HashMap::new());
        LobbyList { list }
    }

    /// Remove any `Lobby` objects that we haven't heard from in awhile.
    ///
    /// # Arguments
    ///
    /// * `expiration_threshold` - The point past which a `Lobby` is considered "stale."
    pub fn cleanup(&self, expiration_threshold: Duration) {
        let to_delete = self
            .list
            .read()
            .unwrap()
            .iter()
            .filter(|(_, lobby)| lobby.modified.elapsed() >= expiration_threshold)
            .map(|(k, _)| *k)
            .collect::<Vec<SocketAddr>>();

        let mut list = self.list.write().unwrap();
        for k in to_delete {
            list.remove(&k);
        }
    }

    /// Create and insert a `Lobby` into this `LobbyList`. If a `Lobby` already exists for the
    /// reported address, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `addr` - The host's address.
    /// * `datagram` - The received `Command::Hello` `Datagram`.
    pub fn insert(&self, addr: &SocketAddr, datagram: &Datagram) {
        let lobby = Lobby::new(addr, datagram);
        self.list.write().unwrap().insert(*addr, lobby);
    }

    /// Remove a `Lobby` from the `LobbyList`.
    ///
    /// # Arguments
    ///
    /// * `addr` - The host's address.
    pub fn remove(&self, addr: &SocketAddr) {
        self.list.write().unwrap().remove(addr);
    }

    /// Returns a vector of `Command::Response` `Datagram`s to respond to a `Command::Query`.
    ///
    /// # Arguments
    ///
    /// * `term` - A search term. Supported by the protocol, but currently has no effect on the results in `ratd`.
    /// * `query_id` - The query ID to which we are responding.
    /// * `limit` - The maximum number of responses to send back.
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
                Some(_term) => list.iter().take(size),
                None => list.iter().take(size),
            };
            for (idx, (_, lobby)) in filtered_list.enumerate() {
                let response_index = idx as u16;
                responses.push(lobby.as_serialized_response(
                    query_id,
                    response_index,
                    response_count,
                ));
            }
        }

        responses
    }
}

#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, Ipv4Addr},
        time::Duration,
    };

    use super::{parse::TryParse, *};

    fn build_hello() -> Datagram {
        let mut datagram = Datagram::new(Command::Hello);
        datagram.add_tag(TrackerTag::SoftwareVersion(RawStringPayload(vec![
            49, 46, 48, 46, 50,
        ])));
        datagram.add_tag(TrackerTag::PlayerLimit(SmallIntPayload(6)));
        datagram.add_tag(TrackerTag::Invitation(RawStringPayload(vec![
            73, 110, 118, 105, 116, 97, 116, 105, 111, 110, 32, 77, 101, 115, 115, 97, 103, 101,
        ])));
        datagram.add_tag(TrackerTag::HasPassword);
        datagram.add_tag(TrackerTag::PlayerIpPort(IndexedSocketAddrPayload(
            PlayerId::new(0),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 2, 15)), 19567),
        )));
        datagram.add_tag(TrackerTag::LevelDirectory(RawStringPayload(vec![
            65, 65, 32, 78, 111, 114, 109, 97, 108,
        ])));
        datagram.add_tag(TrackerTag::LevelName(RawStringPayload(vec![
            67, 111, 114, 111, 109, 111, 114, 97, 110,
        ])));
        datagram.add_tag(TrackerTag::GameStatus(GameStatusPayload(
            GameStatus::Active,
        )));
        datagram.add_tag(TrackerTag::PlayerNick(IndexedRawStringPayload(
            PlayerId::new(0),
            RawStringPayload(vec![115, 105, 108, 118, 101, 114, 102, 111, 120]),
        )));
        datagram.add_tag(TrackerTag::PlayerLocation(IndexedLocationPayload(
            PlayerId::new(0),
            IntPayload(7_233),
            IntPayload(46_424),
        )));
        datagram.add_tag(TrackerTag::PlayerLives(IndexedIntPayload(
            PlayerId::new(0),
            IntPayload(3),
        )));
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
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 2, 16)), 21541);
        let datagram = build_hello();
        let lobby = Lobby::new(&addr, &datagram);
        assert!(lobby.modified.elapsed() < Duration::from_secs(1));

        let mock_response = lobby.as_serialized_response(1, 0, 256);
        let result = Datagram::try_parse(&mock_response);
        assert!(result.is_ok());

        let datagram = result.unwrap();
        assert_eq!(
            Some(SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(10, 0, 2, 16)),
                19567
            )),
            datagram.host_address
        );

        let host_addr = datagram.tags.iter().find(|tag| match tag {
            TrackerTag::PlayerIpPort(IndexedSocketAddrPayload(id, _)) => *id == PlayerId::new(0),
            _ => false,
        });
        assert!(host_addr.is_some());

        if let Some(TrackerTag::PlayerIpPort(IndexedSocketAddrPayload(_, host_addr))) = host_addr {
            assert_eq!(
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 2, 16)), 19567),
                *host_addr
            );
        }
    }

    #[test]
    #[should_panic]
    fn fail_new_lobby() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 2, 15)), 19567);
        let mut datagram = build_hello();
        datagram.command = Command::Goodbye;
        let _ = Lobby::new(&addr, &datagram);
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

    #[test]
    fn lobbylist_cleanup() {
        let lobby_list = LobbyList::new();
        assert_eq!(0, lobby_list.list.read().unwrap().len());

        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 2, 15)), 19567);
        let addr_2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 2, 16)), 19567);
        let datagram = build_hello();
        let duration = Duration::from_secs(30);

        lobby_list.insert(&addr, &datagram);
        lobby_list.insert(&addr_2, &datagram);
        {
            let mut raw_list = lobby_list.list.write().unwrap();
            let lobby = raw_list.get_mut(&addr).unwrap();
            lobby.modified -= duration;
        }
        assert_eq!(2, lobby_list.list.read().unwrap().len());

        lobby_list.cleanup(duration);
        assert_eq!(1, lobby_list.list.read().unwrap().len());
    }
}
