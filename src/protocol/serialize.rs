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
        let raw_value = self.0 as u8;
        vec![raw_value]
    }
}

impl Serialize for GameStatusPayload {
    fn serialize(&self) -> Vec<u8> {
        let raw_value = self.0 as u8;
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

fn pack_tag(id: u8, payload: &impl Serialize) -> Vec<u8> {
    let mut payload = payload.serialize();
    let mut value = Vec::with_capacity(2 + payload.len());
    value.push(id);
    value.push(payload.len() as u8);
    value.append(&mut payload);
    value
}

impl Serialize for TrackerTag {
    fn serialize(&self) -> Vec<u8> {
        match self {
            TrackerTag::Command(payload) => pack_tag(1, payload),
            TrackerTag::QueryID(payload) => pack_tag(2, payload),
            TrackerTag::QueryString(payload) => pack_tag(3, payload),
            TrackerTag::HostDomain(payload) => pack_tag(4, payload),
            TrackerTag::ResponseIndex(payload) => pack_tag(5, payload),
            TrackerTag::ResponseCount(payload) => pack_tag(6, payload),
            TrackerTag::StatusMessage(payload) => pack_tag(7, payload),
            TrackerTag::InfoMessage(payload) => pack_tag(8, payload),
            TrackerTag::Invitation(payload) => pack_tag(9, payload),
            TrackerTag::HasPassword => vec![10, 0],
            TrackerTag::PlayerLimit(payload) => pack_tag(11, payload),
            TrackerTag::GameStatus(payload) => pack_tag(12, payload),
            TrackerTag::LevelDirectory(payload) => pack_tag(13, payload),
            TrackerTag::LevelName(payload) => pack_tag(14, payload),
            TrackerTag::ProtocolVersion(payload) => pack_tag(15, payload),
            TrackerTag::SoftwareVersion(payload) => pack_tag(16, payload),
            TrackerTag::PlayerIPPort(payload) => pack_tag(255, payload),
            TrackerTag::PlayerNick(payload) => pack_tag(254, payload),
            TrackerTag::PlayerLives(payload) => pack_tag(253, payload),
            TrackerTag::PlayerLocation(payload) => pack_tag(252, payload),
        }
    }
}

impl Serialize for Datagram {
    fn serialize(&self) -> Vec<u8> {
        let mut size = 7;
        let mut protocol_version = TrackerTag::ProtocolVersion(IntPayload(self.protocol_version))
            .serialize();
        let mut command = TrackerTag::Command(CommandPayload(self.command))
            .serialize();
        let mut query_id = if let Some(query_id) = self.query_id {
            size += 6;
            TrackerTag::QueryID(BigIntPayload(query_id)).serialize()
        } else {
            vec![]
        };
        let mut value = Vec::with_capacity(size);
        value.append(&mut protocol_version);
        value.append(&mut command);
        value.append(&mut query_id);
        for tag in &self.tags {
            let mut tag = tag.serialize();
            value.append(&mut tag);
        }
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

    #[test]
    fn serialize_trackertag() {
        let value = TrackerTag::Command(CommandPayload(Command::Query));
        assert_eq!(vec![1, 1, 0], value.serialize());
        let value = TrackerTag::QueryID(BigIntPayload(3225));
        assert_eq!(vec![2, 4, 0, 0, 12, 153], value.serialize());
        let value = TrackerTag::QueryString(RawStringPayload(vec![115, 105, 108, 118, 101, 114, 102, 111, 120]));
        assert_eq!(vec![3, 9, 115, 105, 108, 118, 101, 114, 102, 111, 120], value.serialize());
        let value = TrackerTag::HostDomain(RawStringPayload(vec![112, 108, 97, 121, 97, 118, 97, 114, 97, 46, 110, 101, 116]));
        assert_eq!(vec![4, 13, 112, 108, 97, 121, 97, 118, 97, 114, 97, 46, 110, 101, 116], value.serialize());
        let value = TrackerTag::ResponseIndex(IntPayload(499));
        assert_eq!(vec![5, 2, 1, 243], value.serialize());
        let value = TrackerTag::ResponseCount(IntPayload(500));
        assert_eq!(vec![6, 2, 1, 244], value.serialize());
        let value = TrackerTag::StatusMessage(RawStringPayload(vec![82, 101, 97, 100, 121, 46]));
        assert_eq!(vec![7, 6, 82, 101, 97, 100, 121, 46], value.serialize());
        let value = TrackerTag::InfoMessage(RawStringPayload(vec![87, 105, 100, 101, 32, 79, 112, 101, 110, 32, 83, 111, 117, 114, 99, 101, 115]));
        assert_eq!(vec![8, 17, 87, 105, 100, 101, 32, 79, 112, 101, 110, 32, 83, 111, 117, 114, 99, 101, 115], value.serialize());
        let value = TrackerTag::Invitation(RawStringPayload(vec![73, 110, 118, 105, 116, 97, 116, 105, 111, 110, 32, 77, 101, 115, 115, 97, 103, 101]));
        assert_eq!(vec![9, 18, 73, 110, 118, 105, 116, 97, 116, 105, 111, 110, 32, 77, 101, 115, 115, 97, 103, 101], value.serialize());
        let value = TrackerTag::HasPassword;
        assert_eq!(vec![10, 0], value.serialize());
        let value = TrackerTag::PlayerLimit(SmallIntPayload(6));
        assert_eq!(vec![11, 1, 6], value.serialize());
        let value = TrackerTag::GameStatus(GameStatusPayload(GameStatus::Active));
        assert_eq!(vec![12, 1, 2], value.serialize());
        let value = TrackerTag::LevelDirectory(RawStringPayload(vec![65, 65, 32, 78, 111, 114, 109, 97, 108]));
        assert_eq!(vec![13, 9, 65, 65, 32, 78, 111, 114, 109, 97, 108], value.serialize());
        let value = TrackerTag::LevelName(RawStringPayload(vec![67, 111, 114, 111, 109, 111, 114, 97, 110]));
        assert_eq!(vec![14, 9, 67, 111, 114, 111, 109, 111, 114, 97, 110], value.serialize());
        let value = TrackerTag::ProtocolVersion(IntPayload(6));
        assert_eq!(vec![15, 2, 0, 6], value.serialize());
        let value = TrackerTag::SoftwareVersion(RawStringPayload(vec![49, 46, 48, 46, 50]));
        assert_eq!(vec![16, 5, 49, 46, 48, 46, 50], value.serialize());
        let value = TrackerTag::PlayerIPPort(IndexedSocketAddrPayload(
            PlayerId::new(0),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 2, 15)), 19567)
        ));
        assert_eq!(vec![255, 7, 0, 10, 0, 2, 15, 76, 111], value.serialize());
        let value = TrackerTag::PlayerNick(IndexedRawStringPayload(
            PlayerId::new(0),
            RawStringPayload(vec![115, 105, 108, 118, 101, 114, 102, 111, 120])
        ));
        assert_eq!(vec![254, 10, 0, 115, 105, 108, 118, 101, 114, 102, 111, 120], value.serialize());
        let value = TrackerTag::PlayerLives(IndexedIntPayload(PlayerId::new(0), IntPayload(3)));
        assert_eq!(vec![253, 3, 0, 0, 3], value.serialize());
        let value = TrackerTag::PlayerLocation(IndexedLocationPayload(
            PlayerId::new(0),
            IntPayload(7_233),
            IntPayload(46_424)
        ));
        assert_eq!(vec![252, 5, 0, 28, 65, 181, 88], value.serialize());
    }

    #[test]
    fn serialize_query_datagram() {
        let mut value = Datagram::new(Command::Query);
        value.set_query_id(Some(3225));
        value.add_tag(TrackerTag::SoftwareVersion(RawStringPayload(vec![49, 46, 48, 46, 50])));
        value.add_tag(TrackerTag::PlayerLocation(IndexedLocationPayload(
            PlayerId::new(0),
            IntPayload(7_233),
            IntPayload(46_424)
        )));
        value.add_tag(TrackerTag::ResponseCount(IntPayload(500)));
        value.add_tag(TrackerTag::QueryString(RawStringPayload(vec![])));

        let expected = vec![
            15, 2, 0, 6,
            1, 1, 0,
            2, 4, 0, 0, 12, 153,
            16, 5, 49, 46, 48, 46, 50,
            252, 5, 0, 28, 65, 181, 88,
            6, 2, 1, 244,
            3, 0
        ];
        assert_eq!(expected, value.serialize());
    }

    #[test]
    fn serialize_hello_datagram() {
        let mut value = Datagram::new(Command::Hello);
        value.add_tag(TrackerTag::SoftwareVersion(RawStringPayload(vec![49, 46, 48, 46, 50])));
        value.add_tag(TrackerTag::PlayerLimit(SmallIntPayload(6)));
        value.add_tag(TrackerTag::Invitation(RawStringPayload(vec![73, 110, 118, 105, 116, 97, 116, 105, 111, 110, 32, 77, 101, 115, 115, 97, 103, 101])));
        value.add_tag(TrackerTag::HasPassword);
        value.add_tag(TrackerTag::PlayerIPPort(IndexedSocketAddrPayload(
            PlayerId::new(0),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 2, 15)), 19567)
        )));
        value.add_tag(TrackerTag::LevelDirectory(RawStringPayload(vec![65, 65, 32, 78, 111, 114, 109, 97, 108])));
        value.add_tag(TrackerTag::LevelName(RawStringPayload(vec![67, 111, 114, 111, 109, 111, 114, 97, 110])));
        value.add_tag(TrackerTag::GameStatus(GameStatusPayload(GameStatus::Active)));
        value.add_tag(TrackerTag::PlayerNick(IndexedRawStringPayload(
            PlayerId::new(0),
            RawStringPayload(vec![115, 105, 108, 118, 101, 114, 102, 111, 120])
        )));
        value.add_tag(TrackerTag::PlayerLocation(IndexedLocationPayload(
            PlayerId::new(0),
            IntPayload(7_233),
            IntPayload(46_424)
        )));
        value.add_tag(TrackerTag::PlayerLives(IndexedIntPayload(PlayerId::new(0), IntPayload(3))));

        let expected = vec![
            15, 2, 0, 6,
            1, 1, 2,
            16, 5, 49, 46, 48, 46, 50,
            11, 1, 6,
            9, 18, 73, 110, 118, 105, 116, 97, 116, 105, 111, 110, 32, 77, 101, 115, 115, 97, 103, 101,
            10, 0,
            255, 7, 0, 10, 0, 2, 15, 76, 111,
            13, 9, 65, 65, 32, 78, 111, 114, 109, 97, 108,
            14, 9, 67, 111, 114, 111, 109, 111, 114, 97, 110,
            12, 1, 2,
            254, 10, 0, 115, 105, 108, 118, 101, 114, 102, 111, 120,
            252, 5, 0, 28, 65, 181, 88,
            253, 3, 0, 0, 3
        ];
        assert_eq!(expected, value.serialize());
    }
}
