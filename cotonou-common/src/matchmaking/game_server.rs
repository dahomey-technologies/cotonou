use crate::types::GameServerId;
use super::matchmaking_session::SessionId;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub enum GameServerHostType {
    Dynamic = 1,
    Static = 2,
}

impl fmt::Display for GameServerHostType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            GameServerHostType::Dynamic => "Dynamic",
            GameServerHostType::Static => "Static",
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameServer {
    /// Game server unique id (uuid)
    #[serde(rename = "i")]
    pub game_server_id: GameServerId,

    /// Host name
    #[serde(rename = "ii")]
    pub host_name: String,

    /// Host type (0: Undefined, 1: Dynamic, 2: Static)
    #[serde(rename = "ht")]
    pub host_type: GameServerHostType,

    /// Host boot time (unix timestamp)
    #[serde(rename = "hb")]
    pub host_boot_time: u64,

    /// Host provider (AWS, OVH, GPortal, etc.)
    #[serde(rename = "hp")]
    pub host_provider: String,

    /// Game version
    #[serde(rename = "gv")]
    pub game_version: String,

    /// Unix Process id
    #[serde(rename = "pid")]
    pub process_id: u32,

    /// Ip address
    #[serde(rename = "ip")]
    pub ip_address: String,

    /// Port
    #[serde(rename = "p")]
    pub port: u16,

    /// Matchmaking session id
    #[serde(rename = "s")]
    pub session_id: Option<SessionId>,

    /// Last keep alive time (unix timestamp)
    #[serde(rename = "t")]
    pub keep_alive_time: u64,
}

#[cfg(feature = "matchmaking")]
#[cfg(test)]
mod tests {
    use crate::types::{GameServerId, UniqueId};

    const TEST_UUID: &str = "1f6cf4f5d977453394c6ba33b7a3e299";

    #[test]
    fn debug() {
        let server_id: GameServerId = UniqueId::try_parse(TEST_UUID).unwrap().into();
        let str = format!("{server_id:?}");
        assert_eq!(format!("GameServerId({TEST_UUID})"), str);
    }

    #[test]
    fn display() {
        let server_id: GameServerId = UniqueId::try_parse(TEST_UUID).unwrap().into();
        let str = server_id.to_string();
        assert_eq!(TEST_UUID, str);
    }

    #[test]
    fn deserialize() {
        let expected_id: GameServerId = UniqueId::try_parse(TEST_UUID).unwrap().into();
        let actual_id = serde_json::from_str::<GameServerId>(&format!("\"{TEST_UUID}\"")).unwrap();
        assert_eq!(expected_id, actual_id);

        let result = serde_json::from_str::<GameServerId>("\"abc\"");
        println!("{result:?}");
        assert!(result.is_err());
    }

    #[test]
    fn serialize() {
        let expected_id = format!("\"{TEST_UUID}\"");
        let actual_id =
            serde_json::to_string(&GameServerId::from(UniqueId::try_parse(TEST_UUID).unwrap())).unwrap();
        assert_eq!(expected_id, actual_id);
    }
}
