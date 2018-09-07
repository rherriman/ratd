pub mod parser;

use std::net::SocketAddr;

use self::parser::ParseError;

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

impl Command {
    /// Attempt to construct Command via a conversion.
    ///
    /// Essentially an implementation of the TryFrom trait, but without the trait: it hasn't moved
    /// to stable Rust, yet.
    pub fn try_from(i: u8) -> Result<Command, ParseError> {
        let tag = match i {
            0 => Command::Query,
            1 => Command::Response,
            2 => Command::Hello,
            3 => Command::Goodbye,
            _ => return Err(ParseError::InvalidCommand),
        };

        Ok(tag)
    }
}

#[repr(u8)]
#[derive(Debug)]
pub enum GameStatus {
    NotLoaded,
    Loaded,
    Active,
    Paused,
}

impl GameStatus {
    /// Attempt to construct GameStatus via a conversion.
    ///
    /// Essentially an implementation of the TryFrom trait, but without the trait: it hasn't moved
    /// to stable Rust, yet.
    pub fn try_from(i: u8) -> Result<GameStatus, ParseError> {
        let tag = match i {
            0 => GameStatus::NotLoaded,
            1 => GameStatus::Loaded,
            2 => GameStatus::Active,
            3 => GameStatus::Paused,
            _ => return Err(ParseError::InvalidGameStatus),
        };

        Ok(tag)
    }
}

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
pub enum TrackerTag {
    Null,
    Command(Command),
    QueryID(u32),
    QueryString(Vec<u8>),
    HostDomain(Vec<u8>),
    ResponseIndex(u16),
    ResponseCount(u16),
    StatusMessage(Vec<u8>),
    InfoMessage(Vec<u8>),
    Invitation(Vec<u8>),
    HasPassword,
    PlayerLimit(u8),
    GameStatus(GameStatus),
    LevelDirectory(Vec<u8>),
    LevelName(Vec<u8>),
    ProtocolVersion(u16),
    SoftwareVersion(Vec<u8>),

    // (Indexed) Player fields.
    PlayerIPPort(PlayerId, SocketAddr),
    PlayerNick(PlayerId, Vec<u8>),
    PlayerLives(PlayerId, u16),
    PlayerLocation(PlayerId, i16, i16),
}

impl TrackerTag {
    pub const NULL_ID: u8 = 0;

    /// Attempt to construct TrackerTag via a conversion.
    ///
    /// Essentially an implementation of the TryFrom trait, but without the trait: it hasn't moved
    /// to stable Rust, yet.
    pub fn try_from(bytes: &[u8]) -> Result<TrackerTag, ParseError> {
        if bytes.len() < 2 {
            return Err(ParseError::InvalidTag);
        }

        let contents = &bytes[2..];
        if contents.len() != bytes[1] as usize {
            return Err(ParseError::InvalidTag);
        }

        let tag = match bytes[0] {
            TrackerTag::NULL_ID => TrackerTag::Null,
            1 => TrackerTag::Command(parser::try_bytes_to_command(contents)?),
            2 => TrackerTag::QueryID(parser::try_bytes_to_u32(contents)?),
            3 => TrackerTag::QueryString(parser::bytes_to_vec_string(contents)),
            4 => TrackerTag::HostDomain(parser::bytes_to_vec_string(contents)),
            5 => TrackerTag::ResponseIndex(parser::try_bytes_to_u16(contents)?),
            6 => TrackerTag::ResponseCount(parser::try_bytes_to_u16(contents)?),
            7 => TrackerTag::StatusMessage(parser::bytes_to_vec_string(contents)),
            8 => TrackerTag::InfoMessage(parser::bytes_to_vec_string(contents)),
            9 => TrackerTag::Invitation(parser::bytes_to_vec_string(contents)),
            10 => TrackerTag::HasPassword,
            11 => TrackerTag::PlayerLimit(parser::try_bytes_to_u8(contents)?),
            12 => TrackerTag::GameStatus(parser::try_bytes_to_gamestatus(contents)?),
            13 => TrackerTag::LevelDirectory(parser::bytes_to_vec_string(contents)),
            14 => TrackerTag::LevelName(parser::bytes_to_vec_string(contents)),
            15 => TrackerTag::ProtocolVersion(parser::try_bytes_to_u16(contents)?),
            16 => TrackerTag::SoftwareVersion(parser::bytes_to_vec_string(contents)),
            255 => {
                let result = parser::try_bytes_to_indexed_socketaddr(contents)?;
                TrackerTag::PlayerIPPort(result.0, result.1)
            },
            254 => {
                let result = parser::try_bytes_to_indexed_vec_string(contents)?;
                TrackerTag::PlayerNick(result.0, result.1)
            },
            253 => {
                let result = parser::try_bytes_to_indexed_u16(contents)?;
                TrackerTag::PlayerLives(result.0, result.1)
            },
            252 => {
                let result = parser::try_bytes_to_indexed_i16_i16(contents)?;
                TrackerTag::PlayerLocation(result.0, result.1, result.2)
            },
            _ => return Err(ParseError::InvalidTag),
        };

        Ok(tag)
    }
}

#[derive(Debug)]
pub struct Datagram {
    protocol_version: Option<u16>,
    command: Option<Command>,
    tags: Vec<TrackerTag>,
}

impl Datagram {
    pub fn new(command: Command) -> Datagram {
        let tags = Vec::new();
        Datagram { protocol_version: Some(PROTOCOL_VERSION), command: Some(command), tags }
    }

    pub fn try_parse(bytes: &[u8]) -> Result<Datagram, ParseError> {
        let mut protocol_version = None;
        let mut command = None;
        let tags = parser::try_split_tags(bytes)?
            .into_iter()
            .map(|slice| TrackerTag::try_from(slice))
            .filter(|tag| match tag {
                Ok(TrackerTag::ProtocolVersion(v)) => {
                    protocol_version = Some(*v);
                    false
                },
                Ok(TrackerTag::Command(c)) => {
                    command = Some((*c).clone());
                    false
                },
                _ => true
            })
            .collect::<Result<Vec<_>, _>>()?;

        if let None = protocol_version {
            return Err(ParseError::MissingProtocolVersion);
        }

        if let None = command {
            return Err(ParseError::MissingCommand);
        }

        Ok(Datagram { protocol_version, command, tags })
    }

    pub fn add_tag(&mut self, tag: TrackerTag) {
        self.tags.push(tag);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_QUERY: [u8; 33] = [
        15, 2, 0, 6,                // Protocol version
        16, 5, 49, 46, 48, 46, 50,  // Software version
        1, 1, 0,                    // Command (query)
        252, 5, 0, 28, 65, 181, 88, // Location
        2, 4, 0, 0, 12, 153,        // Query ID
        6, 2, 1, 244,               // Response count
        3, 0];                      // Query String
    const TEST_HELLO_SIMPLE: [u8; 68] = [
        15, 2, 0, 6,               // Protocol version
        16, 5, 49, 46, 48, 46, 50, // Software Version
        11, 1, 6,                  // Player limit

        // Invitation message
        9, 18, 73, 110, 118, 105, 116, 97, 116, 105, 111, 110, 32, 77, 101, 115, 115, 97, 103, 101,

        // Address + port of first player
        255, 7, 0, 10, 0, 2, 15, 76, 111,

        // Game status
        12, 1, 0,

        // Nick of first player
        254, 10, 0, 115, 105, 108, 118, 101, 114, 102, 111, 120,

        // Location of first player
        252, 5, 0, 28, 65, 181, 88,

        // Command (in this case, "Hello")
        1, 1, 2
    ];
    const TEST_HELLO_COMPLEX: [u8; 97] = [
        15, 2, 0, 6,               // Protocol version
        16, 5, 49, 46, 48, 46, 50, // Software version
        11, 1, 6,                  // Player limit

        // Invitation message
        9, 18, 73, 110, 118, 105, 116, 97, 116, 105, 111, 110, 32, 77, 101, 115, 115, 97, 103, 101,

        // Has password
        10, 0,

        // Address + port of first player
        255, 7, 0, 10, 0, 2, 15, 76, 111,

        // Level directory
        13, 9, 65, 65, 32, 78, 111, 114, 109, 97, 108,

        // Level name
        14, 9, 67, 111, 114, 111, 109, 111, 114, 97, 110,

        // Game status
        12, 1, 2,

        // Nick of first player
        254, 10, 0, 115, 105, 108, 118, 101, 114, 102, 111, 120,

        // Location of first player
        252, 5, 0, 28, 65, 181, 88,

        // Lives of first player
        253, 3, 0, 0, 3,

        // Command
        1, 1, 2
    ];
    const TEST_GOODBYE: [u8; 97] = [
        15, 2, 0, 6,               // Protocol version
        16, 5, 49, 46, 48, 46, 50, // Software version
        11, 1, 6,                  // Player limit

        // Invitation message
        9, 18, 73, 110, 118, 105, 116, 97, 116, 105, 111, 110, 32, 77, 101, 115, 115, 97, 103, 101,

        // Has password
        10, 0,

        // Address + port of first player
        255, 7, 0, 10, 0, 2, 15, 76, 111,

        // Level directory
        13, 9, 65, 65, 32, 78, 111, 114, 109, 97, 108,

        // Level name
        14, 9, 67, 111, 114, 111, 109, 111, 114, 97, 110,

        // Game status
        12, 1, 3,

        // Nick of first player
        254, 10, 0, 115, 105, 108, 118, 101, 114, 102, 111, 120,

        // Location of first player
        252, 5, 0, 28, 65, 181, 88,

        // Lives of first player
        253, 3, 0, 0, 3,

        // Command
        1, 1, 3
    ];

    #[test]
    fn missing_protocol_version() {
        let bytes = [16, 5, 49, 46, 48, 46, 50, 1, 1, 0, 252, 5, 0, 28, 65, 181, 88,
                     2, 4, 0, 0, 12, 153, 6, 2, 1, 244, 3, 0];
        let datagram = Datagram::try_parse(&bytes);
        assert!(datagram.is_err());
        assert_eq!(ParseError::MissingProtocolVersion, datagram.unwrap_err());
    }

    #[test]
    fn missing_command() {
        let bytes = [15, 2, 0, 6, 16, 5, 49, 46, 48, 46, 50, 252, 5, 0, 28, 65, 181, 88,
                     2, 4, 0, 0, 12, 153, 6, 2, 1, 244, 3, 0];
        let datagram = Datagram::try_parse(&bytes);
        assert!(datagram.is_err());
        assert_eq!(ParseError::MissingCommand, datagram.unwrap_err());
    }

    #[test]
    fn parse_query() {
        let datagram = Datagram::try_parse(&TEST_QUERY);
        assert!(datagram.is_ok());

        let datagram = datagram.unwrap();
        assert_eq!(PROTOCOL_VERSION, datagram.protocol_version.unwrap());
        assert_eq!(Command::Query, datagram.command.unwrap());
        assert_eq!(5, datagram.tags.len());
    }

    #[test]
    fn parse_simple_hello() {
        let datagram = Datagram::try_parse(&TEST_HELLO_SIMPLE);
        assert!(datagram.is_ok());

        let datagram = datagram.unwrap();
        assert_eq!(PROTOCOL_VERSION, datagram.protocol_version.unwrap());
        assert_eq!(Command::Hello, datagram.command.unwrap());
        assert_eq!(7, datagram.tags.len());
    }

    #[test]
    fn parse_complex_hello() {
        let datagram = Datagram::try_parse(&TEST_HELLO_COMPLEX);
        assert!(datagram.is_ok());

        let datagram = datagram.unwrap();
        assert_eq!(PROTOCOL_VERSION, datagram.protocol_version.unwrap());
        assert_eq!(Command::Hello, datagram.command.unwrap());
        assert_eq!(11, datagram.tags.len());
    }

    #[test]
    fn parse_goodbye() {
        let datagram = Datagram::try_parse(&TEST_GOODBYE);
        assert!(datagram.is_ok());

        let datagram = datagram.unwrap();
        assert_eq!(PROTOCOL_VERSION, datagram.protocol_version.unwrap());
        assert_eq!(Command::Goodbye, datagram.command.unwrap());
        assert_eq!(11, datagram.tags.len());
    }
}
