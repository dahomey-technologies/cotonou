use cotonou_common::{matchmaking::matchmaking_ticket::{MatchmakingPlayer, MatchmakingPlayerStatus}, unix_now};

#[derive(Clone)]
pub struct MatchmakingAssembler {}

impl MatchmakingAssembler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn convert_to_matchmaking_player(&self, player: &MatchmakingPlayer, ticket_creation_time: u64) -> MatchmakingPlayer {
        let now = unix_now();

        let mut player = player.clone();
        player.creation_time = ticket_creation_time;
        player.new_status_time = now;
        player.status = MatchmakingPlayerStatus::Matched;
        player.time_until_open_session = unix_now() - ticket_creation_time;
        player.time_until_close_session = 0;

        player
    }
}
