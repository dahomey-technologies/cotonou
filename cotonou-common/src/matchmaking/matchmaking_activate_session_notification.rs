use crate::{matchmaking::SessionId, notifications::Notification};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MatchmakingActivateSessionNotification {
    pub matchmaking_session_id: SessionId,
    pub game_mode: String,
    pub encryption_key: String,
    pub max_players: usize,
    pub team_player_count: usize,
}

#[typetag::serde]
impl Notification for MatchmakingActivateSessionNotification {}
