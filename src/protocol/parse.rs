use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, SocketAddr}
};

use super::{
    MAX_PLAYERS,
    Command,
    GameStatus,
    CommandPayload,
    GameStatusPayload,
    BigIntPayload,
    IntPayload,
    SmallIntPayload,
    RawStringPayload,
    PlayerId,
    IndexedSocketAddrPayload,
    IndexedRawStringPayload,
    IndexedIntPayload,
    IndexedLocationPayload,
    TrackerTag,
    Datagram
};

#[derive(Debug, PartialEq)]
pub enum Error {
    UnexpectedDatagramBoundary = 1,
    MissingProtocolVersion,
    MissingCommand,
    InvalidTag,
    InvalidCommand,
    InvalidGameStatus,
    InvalidPlayerIndex,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::UnexpectedDatagramBoundary =>
                write!(f, "Unexpected datagram boundary encountered"),
            Error::MissingProtocolVersion =>
                write!(f, "Datagram contained no protocol version information"),
            Error::MissingCommand =>
                write!(f, "Datagram contained no command tag"),
            Error::InvalidTag =>
                write!(f, "Invalid tag encountered"),
            Error::InvalidCommand =>
                write!(f, "Invalid command encountered"),
            Error::InvalidGameStatus =>
                write!(f, "Invalid game status encountered"),
            Error::InvalidPlayerIndex =>
                write!(f, "Invalid player index encountered"),
        }
    }
}

pub trait TryParse where Self: Sized {
    fn try_parse(bytes: &[u8]) -> Result<Self, Error>;
}

impl TryParse for CommandPayload {
    fn try_parse(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 1 {
            return Err(Error::InvalidCommand);
        }

        let command = match bytes[0] {
            0 => Command::Query,
            1 => Command::Response,
            2 => Command::Hello,
            3 => Command::Goodbye,
            _ => return Err(Error::InvalidCommand),
        };

        Ok(CommandPayload(command))
    }
}

impl TryParse for GameStatusPayload {
    fn try_parse(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 1 {
            return Err(Error::InvalidGameStatus);
        }

        let game_status = match bytes[0] {
            0 => GameStatus::NotLoaded,
            1 => GameStatus::Loaded,
            2 => GameStatus::Active,
            3 => GameStatus::Paused,
            _ => return Err(Error::InvalidGameStatus),
        };

        Ok(GameStatusPayload(game_status))
    }
}

impl TryParse for BigIntPayload {
    fn try_parse(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 4 {
            return Err(Error::InvalidTag);
        }

        let combined = if cfg!(target_endian = "big") {
            ((u32::from(bytes[3]) << 24) | (u32::from(bytes[2]) << 16) |
             (u32::from(bytes[1]) << 8) | u32::from(bytes[0]))
        } else {
            ((u32::from(bytes[0]) << 24) | (u32::from(bytes[1]) << 16) |
             (u32::from(bytes[2]) << 8) | u32::from(bytes[3]))
        };
        Ok(BigIntPayload(combined))
    }
}

impl TryParse for IntPayload {
    fn try_parse(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 2 {
            return Err(Error::InvalidTag);
        }

        let combined = if cfg!(target_endian = "big") {
            (u16::from(bytes[1]) << 8) | u16::from(bytes[0])
        } else {
            (u16::from(bytes[0]) << 8) | u16::from(bytes[1])
        };
        Ok(IntPayload(combined))
    }
}

impl TryParse for SmallIntPayload {
    fn try_parse(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 1 {
            return Err(Error::InvalidTag);
        }

        Ok(SmallIntPayload(bytes[0]))
    }
}

impl TryParse for RawStringPayload {
    fn try_parse(bytes: &[u8]) -> Result<Self, Error> {
        Ok(RawStringPayload(bytes.to_vec()))
    }
}

impl TryParse for PlayerId {
    fn try_parse(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 1 || bytes[0] >= MAX_PLAYERS {
            return Err(Error::InvalidPlayerIndex);
        }

        Ok(PlayerId::new(bytes[0]))
    }
}

impl TryParse for IndexedSocketAddrPayload {
    fn try_parse(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 7 {
            return Err(Error::InvalidTag);
        }

        let player = PlayerId::try_parse(&bytes[..1])?;
        let ip = IpAddr::V4(Ipv4Addr::new(bytes[1], bytes[2], bytes[3], bytes[4]));
        let port = IntPayload::try_parse(&bytes[5..])?.0;
        Ok(IndexedSocketAddrPayload(player, SocketAddr::new(ip, port)))
    }
}

impl TryParse for IndexedRawStringPayload {
    fn try_parse(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.is_empty() {
            return Err(Error::InvalidTag);
        }

        let player = PlayerId::try_parse(&bytes[..1])?;
        let raw_string = RawStringPayload::try_parse(&bytes[1..])?;
        Ok(IndexedRawStringPayload(player, raw_string))
    }
}

impl TryParse for IndexedIntPayload {
    fn try_parse(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 3 {
            return Err(Error::InvalidTag);
        }

        let player = PlayerId::try_parse(&bytes[..1])?;
        let u16_data = IntPayload::try_parse(&bytes[1..])?;
        Ok(IndexedIntPayload(player, u16_data))
    }
}

impl TryParse for IndexedLocationPayload {
    fn try_parse(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 5 {
            return Err(Error::InvalidTag);
        }

        let player = PlayerId::try_parse(&bytes[..1])?;
        let latitude = IntPayload::try_parse(&bytes[1..3])?;
        let longitude = IntPayload::try_parse(&bytes[3..])?;
        Ok(IndexedLocationPayload(player, latitude, longitude))
    }
}

impl TryParse for TrackerTag {
    fn try_parse(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 2 {
            return Err(Error::InvalidTag);
        }

        let payload = &bytes[2..];
        if payload.len() != bytes[1] as usize {
            return Err(Error::InvalidTag);
        }

        let tag = match bytes[0] {
            1 => TrackerTag::Command(CommandPayload::try_parse(payload)?),
            2 => TrackerTag::QueryID(BigIntPayload::try_parse(payload)?),
            3 => TrackerTag::QueryString(RawStringPayload::try_parse(payload)?),
            4 => TrackerTag::HostDomain(RawStringPayload::try_parse(payload)?),
            5 => TrackerTag::ResponseIndex(IntPayload::try_parse(payload)?),
            6 => TrackerTag::ResponseCount(IntPayload::try_parse(payload)?),
            7 => TrackerTag::StatusMessage(RawStringPayload::try_parse(payload)?),
            8 => TrackerTag::InfoMessage(RawStringPayload::try_parse(payload)?),
            9 => TrackerTag::Invitation(RawStringPayload::try_parse(payload)?),
            10 => TrackerTag::HasPassword,
            11 => TrackerTag::PlayerLimit(SmallIntPayload::try_parse(payload)?),
            12 => TrackerTag::GameStatus(GameStatusPayload::try_parse(payload)?),
            13 => TrackerTag::LevelDirectory(RawStringPayload::try_parse(payload)?),
            14 => TrackerTag::LevelName(RawStringPayload::try_parse(payload)?),
            15 => TrackerTag::ProtocolVersion(IntPayload::try_parse(payload)?),
            16 => TrackerTag::SoftwareVersion(RawStringPayload::try_parse(payload)?),
            255 => TrackerTag::PlayerIPPort(IndexedSocketAddrPayload::try_parse(payload)?),
            254 => TrackerTag::PlayerNick(IndexedRawStringPayload::try_parse(payload)?),
            253 => TrackerTag::PlayerLives(IndexedIntPayload::try_parse(payload)?),
            252 => TrackerTag::PlayerLocation(IndexedLocationPayload::try_parse(payload)?),
            _ => return Err(Error::InvalidTag),
        };

        Ok(tag)
    }
}

impl TryParse for Datagram {
    fn try_parse(bytes: &[u8]) -> Result<Self, Error> {
        let mut protocol_version = None;
        let mut command = None;
        let mut tags = Vec::new();
        let mut start_idx = 0;
        let byte_len = bytes.len();
        while start_idx < byte_len {
            // If this tag is a "null" tag, ignore it and skip to the next byte.
            if bytes[start_idx] == TrackerTag::NULL_ID {
                start_idx += 1;
                continue;
            }

            let len_idx = start_idx + 1;
            if len_idx >= byte_len {
                return Err(Error::UnexpectedDatagramBoundary);
            }

            let tag_len = bytes[len_idx] as usize;
            let rbound = len_idx + tag_len + 1;
            if rbound > byte_len {
                return Err(Error::UnexpectedDatagramBoundary);
            }

            let tag = TrackerTag::try_parse(&bytes[start_idx..rbound])?;
            match tag {
                TrackerTag::ProtocolVersion(IntPayload(vers)) => protocol_version = Some(vers),
                TrackerTag::Command(CommandPayload(comm)) => command = Some(comm),
                _ => tags.push(tag),
            }
            start_idx = rbound;
        }

        let protocol_version = protocol_version.ok_or(Error::MissingProtocolVersion)?;
        let command = command.ok_or(Error::MissingCommand)?;

        Ok(Datagram { protocol_version, command, tags })
    }
}

#[cfg(test)]
mod tests {
    use ::protocol::PROTOCOL_VERSION;
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
    fn parse_commandpayload() {
        let bytes = [0, 1, 2, 3, /* Not valid: */ 4];
        for i in 0..bytes.len() - 1 {
            assert!(CommandPayload::try_parse(&bytes[i..i + 1]).is_ok());
        }
        assert!(CommandPayload::try_parse(&bytes[..2]).is_err());
        assert!(CommandPayload::try_parse(&bytes[4..]).is_err());
    }

    #[test]
    fn parse_gamestatuspayload() {
        let bytes = [0, 1, 2, 3, /* Not valid: */ 4];
        for i in 0..bytes.len() - 1 {
            assert!(GameStatusPayload::try_parse(&bytes[i..i + 1]).is_ok());
        }
        assert!(GameStatusPayload::try_parse(&bytes[..2]).is_err());
        assert!(GameStatusPayload::try_parse(&bytes[4..]).is_err());
    }

    #[test]
    fn parse_bigintpayload() {
        let bytes = [12, 34, 56, 78, 90];

        let result = BigIntPayload::try_parse(&bytes[..4]);
        assert!(result.is_ok());
        let BigIntPayload(result) = result.unwrap();
        assert_eq!(203_569_230, result);

        let result = BigIntPayload::try_parse(&bytes[1..5]);
        assert!(result.is_ok());
        let BigIntPayload(result) = result.unwrap();
        assert_eq!(574_115_418, result);

        let result = BigIntPayload::try_parse(&bytes[..3]);
        assert!(result.is_err());

        let result = BigIntPayload::try_parse(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn parse_intpayload() {
        let bytes = [12, 34, 56];

        let result = IntPayload::try_parse(&bytes[..2]);
        assert!(result.is_ok());
        let IntPayload(result) = result.unwrap();
        assert_eq!(3106, result);

        let result = IntPayload::try_parse(&bytes[1..3]);
        assert!(result.is_ok());
        let IntPayload(result) = result.unwrap();
        assert_eq!(8760, result);

        let result = IntPayload::try_parse(&bytes[..1]);
        assert!(result.is_err());

        let result = IntPayload::try_parse(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn parse_smallintpayload() {
        let bytes = [12, 34];

        let result = SmallIntPayload::try_parse(&bytes[..1]);
        assert!(result.is_ok());
        let SmallIntPayload(result) = result.unwrap();
        assert_eq!(12, result);

        let result = SmallIntPayload::try_parse(&bytes[1..]);
        assert!(result.is_ok());
        let SmallIntPayload(result) = result.unwrap();
        assert_eq!(34, result);

        let result = SmallIntPayload::try_parse(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn parse_rawstringpayload() {
        let bytes = [12, 34, 56, 78, 90];
        let RawStringPayload(result) = RawStringPayload::try_parse(&bytes).unwrap();
        assert_eq!(bytes.len(), result.len());
        for i in 0..bytes.len() {
            assert_eq!(bytes[i], result[i]);
        }
    }

    #[test]
    fn parse_playerid() {
        for i in 0..MAX_PLAYERS {
            assert!(PlayerId::try_parse(&[i]).is_ok());
        }
        assert!(PlayerId::try_parse(&[MAX_PLAYERS]).is_err());
    }

    #[test]
    fn parse_indexedsocketaddrpayload() {
        let bytes = [0, 10, 0, 2, 15, 76, 111, /* Extra byte: */ 0x00];

        let result = IndexedSocketAddrPayload::try_parse(&bytes[..7]);
        assert!(result.is_ok());
        let IndexedSocketAddrPayload(player, addr) = result.unwrap();
        assert_eq!(PlayerId::new(0), player);
        assert_eq!(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 2, 15)), 19567), addr);

        let result = IndexedSocketAddrPayload::try_parse(&bytes);
        assert!(result.is_err());

        let result = IndexedSocketAddrPayload::try_parse(&bytes[..6]);
        assert!(result.is_err());

        let result = IndexedSocketAddrPayload::try_parse(&bytes[..1]);
        assert!(result.is_err());

        let result = IndexedSocketAddrPayload::try_parse(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_indexedrawstringpayload() {
        let bytes = [0, 115, 105, 108, 118, 101, 114, 102, 111, 120];

        let result = IndexedRawStringPayload::try_parse(&bytes);
        assert!(result.is_ok());
        let IndexedRawStringPayload(player, RawStringPayload(raw_string)) = result.unwrap();
        assert_eq!(PlayerId::new(0), player);
        assert_eq!(bytes.len() - 1, raw_string.len());
        for i in 0..raw_string.len() {
            assert_eq!(bytes[i + 1], raw_string[i]);
        }

        let result = IndexedRawStringPayload::try_parse(&bytes[0..1]);
        assert!(result.is_ok());
        let IndexedRawStringPayload(player, RawStringPayload(raw_string)) = result.unwrap();
        assert_eq!(PlayerId::new(0), player);
        assert_eq!(0, raw_string.len());

        let result = IndexedRawStringPayload::try_parse(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_indexedintpayload() {
        let bytes = [0, 1, 2, /* Extra byte: */ 0x00];

        let result = IndexedIntPayload::try_parse(&bytes[..3]);
        assert!(result.is_ok());
        let IndexedIntPayload(player, IntPayload(num)) = result.unwrap();
        assert_eq!(PlayerId::new(0), player);
        assert_eq!(258, num);

        let result = IndexedIntPayload::try_parse(&bytes);
        assert!(result.is_err());

        let result = IndexedIntPayload::try_parse(&bytes[..2]);
        assert!(result.is_err());

        let result = IndexedIntPayload::try_parse(&bytes[..1]);
        assert!(result.is_err());

        let result = IndexedIntPayload::try_parse(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_indexedlocationpayload() {
        let bytes = [0, 28, 65, 181, 88, /* Extra byte: */ 0x00];
        let result = IndexedLocationPayload::try_parse(&bytes[..5]);
        assert!(result.is_ok());
        let IndexedLocationPayload(player, IntPayload(num_1), IntPayload(num_2)) = result.unwrap();
        assert_eq!(PlayerId::new(0), player);
        assert_eq!(7_233, num_1);
        assert_eq!(46_424, num_2);

        let result = IndexedLocationPayload::try_parse(&bytes);
        assert!(result.is_err());

        let result = IndexedLocationPayload::try_parse(&bytes[..4]);
        assert!(result.is_err());

        let result = IndexedLocationPayload::try_parse(&bytes[..1]);
        assert!(result.is_err());

        let result = IndexedLocationPayload::try_parse(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn datagram_missing_protocol_version() {
        let bytes = [16, 5, 49, 46, 48, 46, 50, 1, 1, 0, 252, 5, 0, 28, 65, 181, 88,
                     2, 4, 0, 0, 12, 153, 6, 2, 1, 244, 3, 0];
        let datagram = Datagram::try_parse(&bytes);
        assert!(datagram.is_err());
        assert_eq!(Error::MissingProtocolVersion, datagram.unwrap_err());
    }

    #[test]
    fn datagram_missing_command() {
        let bytes = [15, 2, 0, 6, 16, 5, 49, 46, 48, 46, 50, 252, 5, 0, 28, 65, 181, 88,
                     2, 4, 0, 0, 12, 153, 6, 2, 1, 244, 3, 0];
        let datagram = Datagram::try_parse(&bytes);
        assert!(datagram.is_err());
        assert_eq!(Error::MissingCommand, datagram.unwrap_err());
    }

    #[test]
    fn datagram_ignore_null_tags() {
        let bytes = [15, 2, 0, 6,
                     /* Null tag: */ 0,
                     1, 1, 0,
                     6, 2, 1, 244,
                     /* TWO null tags: */ 0, 0,
                     3, 0];
        let datagram = Datagram::try_parse(&bytes);
        assert!(datagram.is_ok());

        let result = datagram.unwrap();
        assert_eq!(PROTOCOL_VERSION, result.protocol_version);
        assert_eq!(Command::Query, result.command);
        assert_eq!(2, result.tags.len());
    }

    #[test]
    fn parse_query() {
        let datagram = Datagram::try_parse(&TEST_QUERY);
        assert!(datagram.is_ok());

        let datagram = datagram.unwrap();
        assert_eq!(PROTOCOL_VERSION, datagram.protocol_version);
        assert_eq!(Command::Query, datagram.command);
        assert_eq!(5, datagram.tags.len());
    }

    #[test]
    fn parse_simple_hello() {
        let datagram = Datagram::try_parse(&TEST_HELLO_SIMPLE);
        assert!(datagram.is_ok());

        let datagram = datagram.unwrap();
        assert_eq!(PROTOCOL_VERSION, datagram.protocol_version);
        assert_eq!(Command::Hello, datagram.command);
        assert_eq!(7, datagram.tags.len());
    }

    #[test]
    fn parse_complex_hello() {
        let datagram = Datagram::try_parse(&TEST_HELLO_COMPLEX);
        assert!(datagram.is_ok());

        let datagram = datagram.unwrap();
        assert_eq!(PROTOCOL_VERSION, datagram.protocol_version);
        assert_eq!(Command::Hello, datagram.command);
        assert_eq!(11, datagram.tags.len());
    }

    #[test]
    fn parse_goodbye() {
        let datagram = Datagram::try_parse(&TEST_GOODBYE);
        assert!(datagram.is_ok());

        let datagram = datagram.unwrap();
        assert_eq!(PROTOCOL_VERSION, datagram.protocol_version);
        assert_eq!(Command::Goodbye, datagram.command);
        assert_eq!(11, datagram.tags.len());
    }
}
