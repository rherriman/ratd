use std::net::IpAddr;

use super::{
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

pub trait Serialize {
    fn serialize(&self) -> Vec<u8>;
}

impl Serialize for CommandPayload {
    fn serialize(&self) -> Vec<u8> {
        let raw_value = self.0.clone() as u8;
        vec![raw_value]
    }
}

impl Serialize for GameStatusPayload {
    fn serialize(&self) -> Vec<u8> {
        let raw_value = self.0.clone() as u8;
        vec![raw_value]
    }
}

impl Serialize for BigIntPayload {
    fn serialize(&self) -> Vec<u8> {
        let value = if cfg!(target_endian = "little") {
            self.0
        } else {
            self.0.swap_bytes()
        };
        vec![
            (value >> 24) as u8,
            ((value >> 16) & 0xff) as u8,
            ((value >> 8) & 0xff) as u8,
            (value & 0xff) as u8
        ]
    }
}

impl Serialize for IntPayload {
    fn serialize(&self) -> Vec<u8> {
        let value = if cfg!(target_endian = "little") {
            self.0
        } else {
            self.0.swap_bytes()
        };
        vec![(value >> 8) as u8, (value & 0xff) as u8]
    }
}

impl Serialize for SmallIntPayload {
    fn serialize(&self) -> Vec<u8> {
        vec![self.0]
    }
}

impl Serialize for RawStringPayload {
    fn serialize(&self) -> Vec<u8> {
        self.0.clone()
    }
}

impl Serialize for PlayerId {
    fn serialize(&self) -> Vec<u8> {
        vec![self.id]
    }
}

impl Serialize for IndexedSocketAddrPayload {
    fn serialize(&self) -> Vec<u8> {
        let mut value = Vec::with_capacity(7);
        let mut ip = match self.1.ip() {
            IpAddr::V4(ip) => ip.octets().to_vec(),
            IpAddr::V6(ip) => ip.octets().to_vec(),
        };
        let port = if cfg!(target_endian = "little") {
            self.1.port()
        } else {
            self.1.port().swap_bytes()
        };
        let mut port = vec![(port >> 8) as u8, (port & 0xff) as u8];
        value.append(&mut self.0.serialize());
        value.append(&mut ip);
        value.append(&mut port);
        value
    }
}

impl Serialize for IndexedRawStringPayload {
    fn serialize(&self) -> Vec<u8> {
        let size = 1 + (self.1).0.len();
        let mut value = Vec::with_capacity(size);
        value.append(&mut self.0.serialize());
        value.append(&mut self.1.serialize());
        value
    }
}

impl Serialize for IndexedIntPayload {
    fn serialize(&self) -> Vec<u8> {
        let mut value = Vec::with_capacity(3);
        value.append(&mut self.0.serialize());
        value.append(&mut self.1.serialize());
        value
    }
}

impl Serialize for IndexedLocationPayload {
    fn serialize(&self) -> Vec<u8> {
        let mut value = Vec::with_capacity(5);
        value.append(&mut self.0.serialize());
        value.append(&mut self.1.serialize());
        value.append(&mut self.2.serialize());
        value
    }
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, SocketAddr};
    use ::protocol::{Command, GameStatus};
    use super::*;

    #[test]
    fn serialize_commandpayload() {
        let values = vec![
            CommandPayload(Command::Query),
            CommandPayload(Command::Response),
            CommandPayload(Command::Hello),
            CommandPayload(Command::Goodbye)
        ];
        assert_eq!(vec![0], values[0].serialize());
        assert_eq!(vec![1], values[1].serialize());
        assert_eq!(vec![2], values[2].serialize());
        assert_eq!(vec![3], values[3].serialize());
    }

    #[test]
    fn serialize_gamestatuspayload() {
        let values = vec![
            GameStatusPayload(GameStatus::NotLoaded),
            GameStatusPayload(GameStatus::Loaded),
            GameStatusPayload(GameStatus::Active),
            GameStatusPayload(GameStatus::Paused)
        ];
        assert_eq!(vec![0], values[0].serialize());
        assert_eq!(vec![1], values[1].serialize());
        assert_eq!(vec![2], values[2].serialize());
        assert_eq!(vec![3], values[3].serialize());
    }

    #[test]
    fn serialize_bigintpayload() {
        let value = BigIntPayload(3225);
        assert_eq!(vec![0, 0, 12, 153], value.serialize());
    }

    #[test]
    fn serialize_intpayload() {
        let value = IntPayload(500);
        assert_eq!(vec![1, 244], value.serialize());
    }

    #[test]
    fn serialize_smallintpayload() {
        let value = SmallIntPayload(4);
        assert_eq!(vec![4], value.serialize());
    }

    #[test]
    fn serialize_rawstringpayload() {
        let value = RawStringPayload(vec![115, 105, 108, 118, 101, 114, 102, 111, 120]);
        assert_eq!(vec![115, 105, 108, 118, 101, 114, 102, 111, 120], value.serialize());
    }

    #[test]
    fn serialize_playerid() {
        let value = PlayerId::new(3);
        assert_eq!(vec![3], value.serialize());
    }

    #[test]
    fn serialize_indexedsocketaddrpayload() {
        let value = IndexedSocketAddrPayload(
            PlayerId::new(0),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 2, 15)), 19567)
        );
        assert_eq!(vec![0, 10, 0, 2, 15, 76, 111], value.serialize());
    }

    #[test]
    fn serialize_indexedrawstringpayload() {
        let value = IndexedRawStringPayload(
            PlayerId::new(0),
            RawStringPayload(vec![115, 105, 108, 118, 101, 114, 102, 111, 120])
        );
        assert_eq!(vec![0, 115, 105, 108, 118, 101, 114, 102, 111, 120], value.serialize());
    }

    #[test]
    fn serialize_indexedintpayload() {
        let value = IndexedIntPayload(
            PlayerId::new(0),
            IntPayload(258)
        );
        assert_eq!(vec![0, 1, 2], value.serialize());
    }

    #[test]
    fn serialize_indexedlocationpayload() {
        let value = IndexedLocationPayload(
            PlayerId::new(0),
            IntPayload(7_233),
            IntPayload(46_424)
        );
        assert_eq!(vec![0, 28, 65, 181, 88], value.serialize());
    }
}
