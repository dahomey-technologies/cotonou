mod cut_lists_matchmaker;
mod matchmaker_context;
mod mt_cut_lists_matchmaker;
mod simple_list_matchmaker;

use crate::match_functions::{FcFsMatchFunctions, MatchFunctions, MmrMatchFunctions};
use cotonou_common::{
    matchmaking::{matchmaking_session::MatchmakingSession, matchmaking_ticket::MatchmakingTicket},
    models::{GameModeConfig, MatchFunctionsConfig, MatchmakerConfig},
};
use cut_lists_matchmaker::*;
pub use matchmaker_context::*;
use mt_cut_lists_matchmaker::*;
use simple_list_matchmaker::*;

pub fn new_matchmaker(
    region_system_name: &str,
    game_mode_config: GameModeConfig,
) -> Box<dyn Matchmaker> {
    let match_functions = new_match_functions(&game_mode_config);

    match game_mode_config.matchmaker_type {
        MatchmakerConfig::SimpleList => Box::new(SimpleListMatchmaker::new(
            region_system_name,
            game_mode_config,
            match_functions,
        )),
        MatchmakerConfig::CutLists => Box::new(CutListsMatchmaker::new(
            region_system_name,
            game_mode_config,
            match_functions,
        )),
        MatchmakerConfig::MultiThreadedCutLists => Box::new(MultiThreadedCutListsMatchmaker::new(
            region_system_name,
            game_mode_config,
            match_functions,
        )),
    }
}

fn new_match_functions(game_mode_config: &GameModeConfig) -> Box<dyn MatchFunctions> {
    match game_mode_config.match_functions_type {
        MatchFunctionsConfig::FirstComeFirstServed => Box::new(FcFsMatchFunctions),
        MatchFunctionsConfig::Mmr {
            max_mmr_distance,
            waiting_time_weight,
        } => Box::new(MmrMatchFunctions::new(
            max_mmr_distance,
            waiting_time_weight,
        )),
    }
}

pub trait Matchmaker: Send {
    fn insert_ticket(&mut self, ticket: &MatchmakingTicket);
    fn remove_ticket(&mut self, ticket: &MatchmakingTicket);
    fn insert_session(&mut self, session: &MatchmakingSession);
    fn remove_session(&mut self, session: &MatchmakingSession);
    /// From open tickets, add players to existing sessions or create new sessions
    /// # Return
    /// new sessions
    fn process(&mut self, context: &mut MatchmakerContext);
}
