pub mod parse;

use std::net::SocketAddr;

const PROTOCOL_VERSION: u16 = 6;
pub const MAX_PLAYERS: u8 = 6;

#[repr(u8)]
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Query,
    Response,
    Hello,
    Goodbye,
}

#[repr(u8)]
#[derive(Debug)]
pub enum GameStatus {
    NotLoaded,
    Loaded,
    Active,
    Paused,
}

#[derive(Debug)]
pub struct CommandPayload(Command);
#[derive(Debug)]
pub struct GameStatusPayload(GameStatus);
#[derive(Debug)]
pub struct BigIntPayload(u32);
#[derive(Debug)]
pub struct IntPayload(u16);
#[derive(Debug)]
pub struct SmallIntPayload(u8);
#[derive(Debug)]
pub struct RawStringPayload(Vec<u8>);

#[derive(Debug, PartialEq)]
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

#[derive(Debug)]
pub struct IndexedSocketAddrPayload(PlayerId, SocketAddr);
#[derive(Debug)]
pub struct IndexedRawStringPayload(PlayerId, RawStringPayload);
#[derive(Debug)]
pub struct IndexedIntPayload(PlayerId, IntPayload);
#[derive(Debug)]
pub struct IndexedLocationPayload(PlayerId, IntPayload, IntPayload);

#[derive(Debug)]
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
    tags: Vec<TrackerTag>,
}

impl Datagram {
    pub fn new(command: Command) -> Datagram {
        let tags = Vec::new();
        Datagram { protocol_version: PROTOCOL_VERSION, command, tags }
    }

    pub fn add_tag(&mut self, tag: TrackerTag) {
        self.tags.push(tag);
    }
}

#[cfg(test)]
mod tests {
    use super::{PROTOCOL_VERSION, MAX_PLAYERS, Command, PlayerId, TrackerTag, Datagram};

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
        assert_eq!(PROTOCOL_VERSION, datagram.protocol_version);
        assert_eq!(Command::Hello, datagram.command);
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
}
