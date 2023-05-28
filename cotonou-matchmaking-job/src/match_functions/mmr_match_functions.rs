use super::MatchFunctions;
use crate::util::{get_average_mmr, get_average_waiting_time};
use cotonou_common::{matchmaking::matchmaking_ticket::MatchmakingPlayer};

#[derive(Clone)]
pub struct MmrMatchFunctions {
    max_mmr_distance: u32,
    waiting_time_weight: u32,
}

impl MmrMatchFunctions {
    pub fn new(max_mmr_distance: u32, waiting_time_weight: u32) -> Self {
        Self {
            max_mmr_distance,
            waiting_time_weight,
        }
    }
}

impl MatchFunctions for MmrMatchFunctions {
    fn calculate_match(&self, group1: &[MatchmakingPlayer], group2: &[MatchmakingPlayer]) -> bool {
        let distance =
            (get_average_mmr(group1) as i64 - get_average_mmr(group2) as i64).unsigned_abs() as u32;
        let waiting_time = get_average_waiting_time(group1);

        u32::min(
            waiting_time * self.waiting_time_weight,
            self.max_mmr_distance,
        ) > distance
    }
}
