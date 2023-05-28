mod fcfs_march_functions;
mod mmr_match_functions;

pub use fcfs_march_functions::*;
pub use mmr_match_functions::*;

use cotonou_common::{
    matchmaking::{
        matchmaking_session::MatchmakingSession,
        matchmaking_ticket::{MatchmakingPlayer, MatchmakingTicket},
    },
    models::GameModeConfig,
};

pub trait MatchFunctions: Send {
    fn is_ticket_with_session_match(
        &self,
        game_mode_config: &GameModeConfig,
        ticket: &MatchmakingTicket,
        session: &MatchmakingSession,
    ) -> bool {
        if !is_size_player_match(
            game_mode_config,
            ticket.players.iter().chain(session.players.iter()),
        ) {
            return false;
        }

        self.is_player_match(&ticket.players, &session.players)
    }

    //-------------------------------------------------------------------------------------------------
    fn is_ticket_with_ticket_match(
        &self,
        game_mode_config: &GameModeConfig,
        ticket1: &MatchmakingTicket,
        ticket2: &MatchmakingTicket,
    ) -> bool {
        if !is_size_player_match(
            game_mode_config,
            ticket1.players.iter().chain(ticket2.players.iter()),
        ) {
            return false;
        }

        self.is_player_match(&ticket1.players, &ticket2.players)
    }

    //-------------------------------------------------------------------------------------------------
    fn is_ticket_match(
        &self,
        game_mode_config: &GameModeConfig,
        ticket: &MatchmakingTicket,
    ) -> bool {
        is_size_player_match(game_mode_config, ticket.players.iter())
    }

    //-------------------------------------------------------------------------------------------------
    fn is_player_match(&self, group1: &[MatchmakingPlayer], group2: &[MatchmakingPlayer]) -> bool;
}

fn is_size_player_match<'b, I: Iterator<Item = &'b MatchmakingPlayer>>(
    game_mode_config: &GameModeConfig,
    players: I,
) -> bool {
    let num_players = players.count();
    num_players >= game_mode_config.min_players && num_players <= game_mode_config.max_players
}

