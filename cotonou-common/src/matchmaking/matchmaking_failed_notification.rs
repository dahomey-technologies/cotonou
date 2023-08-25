use crate::{notifications::Notification, types::ProfileId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum MatchmakingFailureReason {
    CancelledByFriend = 1,
    CancelledByMatchmakingService = 2,
    PrivateServerNotFound = 3,
    PrivateServerFull = 4,
    PrivateServerClosed = 5,
    ServerSessionClosed = 6,
    ExpiredInvitation = 7,
    InvalidInvitation = 8,
}

#[derive(Serialize, Deserialize)]
pub struct MatchmakingFailedNotification {
    pub onwer_profile_id: ProfileId,
    pub failure_reason: MatchmakingFailureReason,
}

#[typetag::serde]
impl Notification for MatchmakingFailedNotification {}
