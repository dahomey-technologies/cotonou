#[cfg(feature = "matchmaking")]
use crate::models::GameRegion;

#[cfg(feature = "matchmaking")]
#[derive(Debug, Clone)]
pub struct GameModeConfig {
    pub name: String,
    pub short_name: String,
    pub matchmaker_type: MatchmakerConfig,
    pub match_functions_type: MatchFunctionsConfig,
    pub min_players: usize,
    pub max_players: usize,
    pub team_player_count: usize,
}

#[cfg(feature = "matchmaking")]
#[derive(Debug, Clone)]
pub enum MatchmakerConfig {
    SimpleList,
    CutLists,
}

#[cfg(feature = "matchmaking")]
#[derive(Debug, Clone)]
pub enum MatchFunctionsConfig {
    FirstComeFirstServed,
    Mmr {
        max_mmr_distance: u32,
        waiting_time_weight: u32
    },
}

#[cfg(feature = "matchmaking")]
#[derive(Debug, Clone)]
pub struct MatchmakingSettings {
    pub game_mode_configs: Vec<GameModeConfig>,
    pub reserved_player_session_timeout: u64,
    pub supported_regions: Vec<GameRegion>,
}
