use crate::{
    matchmaking::MatchmakingPlayer,
    types::{GameServerId, UniqueId},
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum MatchmakingSessionStatus {
    /// Session created with the minimum number of players
    Created,
    /// Session activation has been send to a server
    Activating,
    /// Server has confirmed activation
    Active,
}

/// Matchmaking session
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MatchmakingSession {
    /// Matchmaking unique session id (uuid)
    #[serde(rename = "i")]
    pub session_id: SessionId,

    /// game mode name as defined in matchmaking settings
    #[serde(rename = "m")]
    pub game_mode: String,

    /// Players present in this session
    #[serde(rename = "p")]
    pub players: Vec<MatchmakingPlayer>,

    /// Session creation time (unix timestamp)
    #[serde(rename = "t")]
    pub creation_time: u64,

    /// Session status (0: Undefined, 1: Activating, 2: Active)
    #[serde(rename = "s")]
    pub status: MatchmakingSessionStatus,

    /// Is the session open or closed (lobby or actual game)
    #[serde(rename = "o")]
    pub is_open: bool,

    /// Id of the game server hosting the session
    #[serde(rename = "gs")]
    pub game_server_id: Option<GameServerId>,

    /// Ip address of the game server hosting the session
    #[serde(rename = "ip")]
    pub ip_address: String,

    /// Port of the game server hosting the session
    #[serde(rename = "pt")]
    pub port: u16,

    /// Encryption key for communication between the game server and the game client
    #[serde(rename = "k")]
    pub encryption_key: String,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct SessionId(UniqueId);

impl SessionId {
    pub fn new() -> Self {
        Self(UniqueId::new())
    }

    pub fn try_parse(input: &str) -> Option<Self> {
        UniqueId::try_parse(input).map(Self)
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "redis")]
impl rustis::resp::ToArgs for SessionId {
    fn write_args(&self, args: &mut rustis::resp::CommandArgs) {
        args.arg(self.0);
    }
}

#[cfg(feature = "redis")]
impl rustis::resp::SingleArg for SessionId {}

#[cfg(feature = "redis")]
impl rustis::resp::PrimitiveResponse for SessionId {}
