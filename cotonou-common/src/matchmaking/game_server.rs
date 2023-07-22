use super::matchmaking_session::SessionId;
use crate::models::UniqueId;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
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

pub type HostName = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct GameServer {
    /// Game server unique id (uuid)
    #[serde(rename = "i")]
    pub game_server_id: GameServerId,

    /// Host name
    #[serde(rename = "ii")]
    pub host_name: HostName,

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

    /// Host shutdown request time (unix timestamp)
    #[serde(rename = "hs")]
    pub host_shutdown_request_time: u64,
}

impl GameServer {
    #[inline]
    pub fn is_busy(&self) -> bool {
        self.session_id.is_some()
    }

    #[inline]
    pub fn is_idle(&self) -> bool {
        self.session_id.is_none()
    }

    #[inline]
    pub fn is_stopping(&self) -> bool {
        self.host_shutdown_request_time != 0
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct GameServerId(UniqueId);

impl GameServerId {
    pub fn new() -> Self {
        Self(UniqueId::new())
    }

    pub fn try_parse(input: &str) -> Option<Self> {
        UniqueId::try_parse(input).map(Self)
    }
}

impl Default for GameServerId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for GameServerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "redis")]
impl rustis::resp::ToArgs for GameServerId {
    fn write_args(&self, args: &mut rustis::resp::CommandArgs) {
        args.arg(self.0);
    }
}

#[cfg(feature = "redis")]
impl rustis::resp::SingleArg for GameServerId {}

#[cfg(feature = "redis")]
impl rustis::resp::PrimitiveResponse for GameServerId {}

#[cfg(feature = "matchmaking")]
#[cfg(test)]
mod tests {
    use super::GameServerId;
    use crate::models::UniqueId;

    const TEST_UUID: &str = "1f6cf4f5d977453394c6ba33b7a3e299";

    #[test]
    fn debug() {
        let server_id = GameServerId(UniqueId::try_parse(TEST_UUID).unwrap());
        let str = format!("{server_id:?}");
        assert_eq!(format!("GameServerId({TEST_UUID})"), str);
    }

    #[test]
    fn display() {
        let server_id = GameServerId(UniqueId::try_parse(TEST_UUID).unwrap());
        let str = server_id.to_string();
        assert_eq!(TEST_UUID, str);
    }

    #[test]
    fn deserialize() {
        let expected_id = GameServerId(UniqueId::try_parse(TEST_UUID).unwrap());
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
            serde_json::to_string(&GameServerId(UniqueId::try_parse(TEST_UUID).unwrap())).unwrap();
        assert_eq!(expected_id, actual_id);
    }
}
