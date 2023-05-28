use super::MatchFunctions;
use cotonou_common::matchmaking::matchmaking_ticket::MatchmakingPlayer;

/// First come, first served
pub struct FcFsMatchFunctions;

impl MatchFunctions for FcFsMatchFunctions {
    fn is_player_match(&self, _group1: &[MatchmakingPlayer], _group2: &[MatchmakingPlayer]) -> bool {
        true
    }
}
