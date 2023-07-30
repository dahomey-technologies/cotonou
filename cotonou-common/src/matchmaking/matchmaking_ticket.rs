use super::matchmaking_session::SessionId;
use crate::profile::ProfileId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum MatchmakingPlayerStatus {
    /// Player to match
    Created,
    /// Player matched to a session
    Matched,
    /// Player activation has been send to a server
    Activating,
    /// Server has confirmed activation
    Active,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MatchmakingPlayer {
    #[serde(rename = "i")]
    pub profile_id: ProfileId,
    #[serde(rename = "dn")]
    pub display_name: String,
    #[serde(rename = "mmr")]
    pub mmr: u32,
    #[serde(rename = "latency")]
    pub latency: u32,
    #[serde(rename = "t")]
    pub new_status_time: u64,
    #[serde(rename = "s")]
    pub status: MatchmakingPlayerStatus,
    #[serde(rename = "ct")]
    pub creation_time: u64,
    #[serde(rename = "tos")]
    pub time_until_open_session: u64,
    #[serde(rename = "tcs")]
    pub time_until_close_session: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MatchmakingTicket {
    #[serde(rename = "i")]
    pub owner_profile_id: ProfileId,
    #[serde(rename = "m")]
    pub game_mode: String,
    #[serde(rename = "p")]
    pub players: Vec<MatchmakingPlayer>,
    #[serde(rename = "t")]
    pub creation_time: u64,
    #[serde(rename = "n")]
    pub session_id: Option<SessionId>,
    #[serde(rename = "f")]
    pub servers_full_notification_last_time_sent: u64,
}
