use cotonou_common::{
    models::ProfileId,
    notifications::notification::Notification,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MatchmakingStartedNotification {
    pub owner_profile_id: ProfileId,
    pub region_system_name: String,
    pub game_mode: String,
}

#[typetag::serde]
impl Notification for MatchmakingStartedNotification {}
