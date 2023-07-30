use crate::notifications::Notification;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MatchmakingServersFullNotification {
    pub position_in_queue: usize,
    pub estimated_wait_time: u64,
}

#[typetag::serde]
impl Notification for MatchmakingServersFullNotification {}
