use crate::{
    notifications::notification::Notification,
};
use serde::{Deserialize, Serialize};

use super::matchmaking_session::SessionId;

#[derive(Serialize, Deserialize)]
pub struct MatchmakingCompletedNotification {
    pub matchmaking_session_id: SessionId,
    pub game_mode: String,
    pub ip_address: String,
    pub port: u16,
    pub encryption_key: String,
}

#[typetag::serde]
impl Notification for MatchmakingCompletedNotification {}
