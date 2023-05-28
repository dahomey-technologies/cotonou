use super::MatchFunctions;
use cotonou_common::{matchmaking::matchmaking_ticket::MatchmakingPlayer, unix_now};

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
    fn is_player_match(&self, group1: &[MatchmakingPlayer], group2: &[MatchmakingPlayer]) -> bool {
        let distance =
            (get_average_mmr(group1) as i64 - get_average_mmr(group2) as i64).unsigned_abs() as u32;
        let waiting_time = get_average_waiting_time(group1);

        u32::min(
            waiting_time * self.waiting_time_weight,
            self.max_mmr_distance,
        ) > distance
    }
}

fn get_average_mmr(players: &[MatchmakingPlayer]) -> u32 {
    let sum = players.iter().fold(0, |acc, p| acc + p.mmr);
    sum / players.len() as u32
}

fn get_average_waiting_time(players: &[MatchmakingPlayer]) -> u32 {
    let now = unix_now();
    let sum = players.iter().fold(0, |acc, p| acc + now - p.creation_time);
    sum as u32 / players.len() as u32
}
