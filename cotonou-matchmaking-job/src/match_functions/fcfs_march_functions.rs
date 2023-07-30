use crate::match_functions::MatchFunctions;
use cotonou_common::matchmaking::MatchmakingPlayer;

/// First come, first served
#[derive(Clone)]
pub struct FcFsMatchFunctions;

impl MatchFunctions for FcFsMatchFunctions {
    fn calculate_match(&self, _group1: &[MatchmakingPlayer], _group2: &[MatchmakingPlayer]) -> bool {
        true
    }
}
