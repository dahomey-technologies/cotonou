use crate::{notifications::notification::Notification, models::ProfileId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum MatchmakingFailureReason {
    CancelledByFriend = 1,
    CancelledByMatchmakingService = 2,
    PrivateServerNotFound = 3,
    PrivateServerFull = 4,
    PrivateServerClosed = 5,
    PrivateServerSDSlotFull = 6,
    ServerSessionClosed = 7,
    ExpiredInvitation = 8,
    InvalidInvitation = 9,
}

#[derive(Serialize, Deserialize)]
pub struct MatchmakingFailedNotification {
    pub onwer_profile_id: ProfileId,
    pub failure_reason: MatchmakingFailureReason,
}

#[typetag::serde]
impl Notification for MatchmakingFailedNotification {}
