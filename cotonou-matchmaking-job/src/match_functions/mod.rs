mod fcfs_march_functions;
mod mmr_match_functions;

pub use fcfs_march_functions::*;
pub use mmr_match_functions::*;

use cotonou_common::{
    matchmaking::{
        matchmaking_ticket::{MatchmakingPlayer},
    },
    models::GameModeConfig,
};

pub trait MatchFunctions: MatchFunctionsClone + Send {
    fn is_match(
        &self,
        game_mode_config: &GameModeConfig,
        group1: &[MatchmakingPlayer],
        group2: &[MatchmakingPlayer],
    ) -> bool {
        if !is_in_bounds(game_mode_config, group1.iter().chain(group2.iter())) {
            return false;
        }

        self.calculate_match(group1, group2)
    }

    fn calculate_match(&self, group1: &[MatchmakingPlayer], group2: &[MatchmakingPlayer]) -> bool;
}

pub fn is_in_bounds<'b, I: Iterator<Item = &'b MatchmakingPlayer>>(
    game_mode_config: &GameModeConfig,
    players: I,
) -> bool {
    let num_players = players.count();
    num_players >= game_mode_config.min_players && num_players <= game_mode_config.max_players
}

/// cf. https://stackoverflow.com/questions/30353462/how-to-clone-a-struct-storing-a-boxed-trait-object
pub trait MatchFunctionsClone {
    fn clone_box(&self) -> Box<dyn MatchFunctions>;
}

impl<T> MatchFunctionsClone for T
where
    T: 'static + MatchFunctions + Clone,
{
    fn clone_box(&self) -> Box<dyn MatchFunctions> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn MatchFunctions> {
    fn clone(&self) -> Box<dyn MatchFunctions> {
        self.clone_box()
    }
}
