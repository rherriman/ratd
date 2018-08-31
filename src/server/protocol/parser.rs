use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, SocketAddr}
};

use super::{Command, GameStatus, TrackerTag};

const MAX_PLAYERS: u8 = 6;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedDatagramBoundary = 1,
    MissingProtocolVersion,
    MissingCommand,
    InvalidTag,
    InvalidCommand,
    InvalidGameStatus,
    InvalidPlayerIndex,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::UnexpectedDatagramBoundary =>
                write!(f, "Unexpected datagram boundary encountered"),
            ParseError::MissingProtocolVersion =>
                write!(f, "Datagram contained no protocol version information"),
            ParseError::MissingCommand =>
                write!(f, "Datagram contained no command tag"),
            ParseError::InvalidTag =>
                write!(f, "Invalid tag encountered"),
            ParseError::InvalidCommand =>
                write!(f, "Invalid command encountered"),
            ParseError::InvalidGameStatus =>
                write!(f, "Invalid game status encountered"),
            ParseError::InvalidPlayerIndex =>
                write!(f, "Invalid player index encountered"),
        }
    }
}

pub fn bytes_to_vec_string(bytes: &[u8]) -> Vec<u8> {
    bytes.to_vec()
}

pub fn try_bytes_to_command(bytes: &[u8]) -> Result<Command, ParseError> {
    if bytes.len() > 1 {
        return Err(ParseError::InvalidCommand);
    }

    Command::try_from(bytes[0])
}

pub fn try_bytes_to_gamestatus(bytes: &[u8]) -> Result<GameStatus, ParseError> {
    if bytes.len() > 1 {
        return Err(ParseError::InvalidGameStatus);
    }

    GameStatus::try_from(bytes[0])
}

pub fn try_bytes_to_u8(bytes: &[u8]) -> Result<u8, ParseError> {
    if bytes.len() != 1 {
        return Err(ParseError::InvalidTag);
    }

    Ok(bytes[0])
}

pub fn try_bytes_to_u16(bytes: &[u8]) -> Result<u16, ParseError> {
    if bytes.len() != 2 {
        return Err(ParseError::InvalidTag);
    }

    let combined = if cfg!(target_endian = "big") {
        ((bytes[1] as u16) << 8) | (bytes[0] as u16)
    } else {
        ((bytes[0] as u16) << 8) | (bytes[1] as u16)
    };
    Ok(combined)
}

pub fn try_bytes_to_u32(bytes: &[u8]) -> Result<u32, ParseError> {
    if bytes.len() != 4 {
        return Err(ParseError::InvalidTag);
    }

    let combined = if cfg!(target_endian = "big") {
        (((bytes[3] as u32) << 24) | ((bytes[2] as u32) << 16) |
         ((bytes[1] as u32) << 8) | (bytes[0] as u32))
    } else {
        (((bytes[0] as u32) << 24) | ((bytes[1] as u32) << 16) |
         ((bytes[2] as u32) << 8) | (bytes[3] as u32))
    };
    Ok(combined)
}

pub fn try_byte_to_player_index(i: u8) -> Result<u8, ParseError> {
    if i >= MAX_PLAYERS {
        Err(ParseError::InvalidPlayerIndex)
    } else {
        Ok(i)
    }
}

pub fn try_bytes_to_indexed_socketaddr(bytes: &[u8]) -> Result<(u8, SocketAddr), ParseError> {
    if bytes.len() != 7 {
        return Err(ParseError::InvalidTag);
    }

    let player_idx = try_byte_to_player_index(bytes[0])?;
    let ip = IpAddr::V4(Ipv4Addr::new(bytes[1], bytes[2], bytes[3], bytes[4]));
    let port = try_bytes_to_u16(&bytes[5..])?;
    Ok((player_idx, SocketAddr::new(ip, port)))
}

pub fn try_bytes_to_indexed_vec_string(bytes: &[u8]) -> Result<(u8, Vec<u8>), ParseError> {
    if bytes.len() == 0 {
        return Err(ParseError::InvalidTag);
    }

    let player_idx = try_byte_to_player_index(bytes[0])?;
    let string_data = bytes_to_vec_string(&bytes[1..]);
    Ok((player_idx, string_data))
}

pub fn try_bytes_to_indexed_u16(bytes: &[u8]) -> Result<(u8, u16), ParseError> {
    if bytes.len() != 3 {
        return Err(ParseError::InvalidTag);
    }

    let player_idx = try_byte_to_player_index(bytes[0])?;
    let u16_data = try_bytes_to_u16(&bytes[1..])?;
    Ok((player_idx, u16_data))
}

pub fn try_bytes_to_indexed_i16_i16(bytes: &[u8]) -> Result<(u8, i16, i16), ParseError> {
    if bytes.len() != 5 {
        return Err(ParseError::InvalidTag);
    }

    let player_idx = try_byte_to_player_index(bytes[0])?;
    let latitude = try_bytes_to_u16(&bytes[1..3])? as i16;
    let longitude = try_bytes_to_u16(&bytes[3..])? as i16;
    Ok((player_idx, latitude, longitude))
}

pub fn try_split_tags<'a>(bytes: &'a [u8]) -> Result<Vec<&'a [u8]>, ParseError> {
    let mut v = Vec::new();
    let mut start_idx = 0;
    let byte_len = bytes.len();
    while start_idx < byte_len {
        // If this tag is a TrackerTag::Null, ignore it and skip to the next byte.
        if bytes[start_idx] == TrackerTag::NULL_ID {
            start_idx += 1;
            continue;
        }

        let len_idx = start_idx + 1;
        if len_idx >= byte_len {
            return Err(ParseError::UnexpectedDatagramBoundary);
        }

        let tag_len = bytes[len_idx] as usize;
        let rbound = len_idx + tag_len + 1;
        if rbound > byte_len {
            return Err(ParseError::UnexpectedDatagramBoundary);
        }

        v.push(&bytes[start_idx..rbound]);
        start_idx = rbound;
    }

    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_to_player_index() {
        for i in 0..MAX_PLAYERS {
            assert!(try_byte_to_player_index(i).is_ok());
        }
        assert!(try_byte_to_player_index(MAX_PLAYERS).is_err());
    }

    #[test]
    fn bytes_to_u8() {
        let bytes = [12, 34];
        let result_1 = try_bytes_to_u8(&bytes[0..1]);
        let result_2 = try_bytes_to_u8(&bytes[1..]);
        let result_3 = try_bytes_to_u8(&bytes);
        assert!(result_1.is_ok());
        assert_eq!(12, result_1.unwrap());
        assert!(result_2.is_ok());
        assert_eq!(34, result_2.unwrap());
        assert!(result_3.is_err());
    }

    #[test]
    fn bytes_to_u16() {
        let bytes = [12, 34, 56];
        let result_1 = try_bytes_to_u16(&bytes[0..2]);
        let result_2 = try_bytes_to_u16(&bytes[1..3]);
        let result_3 = try_bytes_to_u16(&bytes[0..1]);
        let result_4 = try_bytes_to_u16(&bytes);
        assert!(result_1.is_ok());
        assert_eq!(3106, result_1.unwrap());
        assert!(result_2.is_ok());
        assert_eq!(8760, result_2.unwrap());
        assert!(result_3.is_err());
        assert!(result_4.is_err());
    }

    #[test]
    fn bytes_to_u32() {
        let bytes = [12, 34, 56, 78, 90];
        let result_1 = try_bytes_to_u32(&bytes[0..4]);
        let result_2 = try_bytes_to_u32(&bytes[1..5]);
        let result_3 = try_bytes_to_u32(&bytes[0..3]);
        let result_4 = try_bytes_to_u32(&bytes);
        assert!(result_1.is_ok());
        assert_eq!(203_569_230, result_1.unwrap());
        assert!(result_2.is_ok());
        assert_eq!(574_115_418, result_2.unwrap());
        assert!(result_3.is_err());
        assert!(result_4.is_err());
    }

    #[test]
    fn bytes_to_vec_string() {
        let bytes = [12, 34, 56, 78, 90];
        let result = super::bytes_to_vec_string(&bytes);
        assert_eq!(bytes.len(), result.len());
        for i in 0..bytes.len() {
            assert_eq!(bytes[i], result[i]);
        }
    }

    #[test]
    fn bytes_to_command() {
        let bytes = [0, 1, 2, 3, /* Not valid: */ 4];
        for i in 0..bytes.len() - 1 {
            assert!(try_bytes_to_command(&bytes[i..i + 1]).is_ok());
        }
        assert!(try_bytes_to_command(&bytes[0..2]).is_err());
        assert!(try_bytes_to_command(&bytes[4..]).is_err());
    }

    #[test]
    fn bytes_to_gamestatus() {
        let bytes = [0, 1, 2, 3, /* Not valid: */ 4];
        for i in 0..bytes.len() - 1 {
            assert!(try_bytes_to_gamestatus(&bytes[i..i + 1]).is_ok());
        }
        assert!(try_bytes_to_gamestatus(&bytes[0..2]).is_err());
        assert!(try_bytes_to_gamestatus(&bytes[4..]).is_err());
    }

    #[test]
    fn bytes_to_indexed_socketaddr() {
        let bytes = [0, 10, 0, 2, 15, 76, 111, /* Extra byte: */ 0x00];

        let result = try_bytes_to_indexed_socketaddr(&bytes[0..7]);
        assert!(result.is_ok());
        let (player_idx, addr) = result.unwrap();
        assert_eq!(0, player_idx);
        assert_eq!(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 2, 15)), 19567), addr);

        let result = try_bytes_to_indexed_socketaddr(&bytes);
        assert!(result.is_err());

        let result = try_bytes_to_indexed_socketaddr(&bytes[0..6]);
        assert!(result.is_err());

        let result = try_bytes_to_indexed_socketaddr(&bytes[0..1]);
        assert!(result.is_err());

        let result = try_bytes_to_indexed_socketaddr(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn bytes_to_indexed_vec_string() {
        let bytes = [0, 115, 105, 108, 118, 101, 114, 102, 111, 120];

        let result = try_bytes_to_indexed_vec_string(&bytes);
        assert!(result.is_ok());
        let (player_idx, vec_string) = result.unwrap();
        assert_eq!(0, player_idx);
        assert_eq!(bytes.len() - 1, vec_string.len());
        for i in 0..vec_string.len() {
            assert_eq!(bytes[i + 1], vec_string[i]);
        }

        let result = try_bytes_to_indexed_vec_string(&bytes[0..1]);
        assert!(result.is_ok());
        let (player_idx, vec_string) = result.unwrap();
        assert_eq!(0, player_idx);
        assert_eq!(0, vec_string.len());

        let result = try_bytes_to_indexed_vec_string(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn bytes_to_indexed_u16() {
        let bytes = [0, 1, 2, /* Extra byte: */ 0x00];

        let result = try_bytes_to_indexed_u16(&bytes[0..3]);
        assert!(result.is_ok());
        let (player_idx, num) = result.unwrap();
        assert_eq!(0, player_idx);
        assert_eq!(258, num);

        let result = try_bytes_to_indexed_u16(&bytes);
        assert!(result.is_err());

        let result = try_bytes_to_indexed_u16(&bytes[0..2]);
        assert!(result.is_err());

        let result = try_bytes_to_indexed_u16(&bytes[0..1]);
        assert!(result.is_err());

        let result = try_bytes_to_indexed_u16(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn bytes_to_indexed_i16_i16() {
        let bytes = [0, 28, 65, 181, 88, /* Extra byte: */ 0x00];
        let result = try_bytes_to_indexed_i16_i16(&bytes[0..5]);
        assert!(result.is_ok());
        let (player_idx, num_1, num_2) = result.unwrap();
        assert_eq!(0, player_idx);
        assert_eq!(7_233, num_1);
        assert_eq!(-19_112, num_2);

        let result = try_bytes_to_indexed_i16_i16(&bytes);
        assert!(result.is_err());

        let result = try_bytes_to_indexed_i16_i16(&bytes[0..4]);
        assert!(result.is_err());

        let result = try_bytes_to_indexed_i16_i16(&bytes[0..1]);
        assert!(result.is_err());

        let result = try_bytes_to_indexed_i16_i16(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn tag_splitting() {
        let bytes = [15, 2, 0, 6, 1, 1, 0, 6, 2, 1, 244, 3, 0];
        let result = try_split_tags(&bytes);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(4, result.len());
        assert_eq!(4, result[0].len());
        assert_eq!(3, result[1].len());
        assert_eq!(4, result[2].len());
        assert_eq!(2, result[3].len());
    }

    #[test]
    fn ignore_null_tags() {
        let bytes = [15, 2, 0, 6,
                     /* Null tag: */ 0,
                     1, 1, 0,
                     6, 2, 1, 244,
                     /* TWO null tags: */ 0, 0,
                     3, 0];
        let result = try_split_tags(&bytes);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(4, result.len());
        assert_eq!(15, result[0][0]);
        assert_eq!(1, result[1][0]);
        assert_eq!(6, result[2][0]);
        assert_eq!(3, result[3][0]);
    }
}
