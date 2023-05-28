use cotonou_common::{matchmaking::matchmaking_ticket::MatchmakingPlayer, unix_now};

pub fn get_average_mmr(players: &[MatchmakingPlayer]) -> u32 {
    let sum = players.iter().fold(0, |acc, p| acc + p.mmr);
    sum / players.len() as u32
}

pub fn get_average_waiting_time(players: &[MatchmakingPlayer]) -> u32 {
    let now = unix_now();
    let sum = players.iter().fold(0, |acc, p| acc + now - p.creation_time);
    sum as u32 / players.len() as u32
}
