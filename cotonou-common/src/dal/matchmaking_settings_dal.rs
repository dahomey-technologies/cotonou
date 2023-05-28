use crate::models::{
    GameModeConfig, GameRegion, MatchFunctionsConfig, MatchmakerConfig, MatchmakingSettings,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct MatchmakingSettingsDAL {
    matchmaking_settings: Arc<MatchmakingSettings>,
}

impl MatchmakingSettingsDAL {
    pub fn new() -> Self {
        Self {
            matchmaking_settings: Arc::new(MatchmakingSettings {
                game_mode_configs: vec![
                    GameModeConfig {
                        name: "QuickMatch".to_owned(),
                        short_name: "qm".to_owned(),
                        matchmaker_type: MatchmakerConfig::SimpleList,
                        match_functions_type: MatchFunctionsConfig::FirstComeFirstServed,
                        min_players: 2,
                        max_players: 8,
                        team_player_count: 4,
                    },
                    GameModeConfig {
                        name: "Ranked".to_owned(),
                        short_name: "r".to_owned(),
                        matchmaker_type: MatchmakerConfig::CutLists,
                        match_functions_type: MatchFunctionsConfig::Mmr {
                            max_mmr_distance: 300,
                            waiting_time_weight: 10,
                        },
                        min_players: 2,
                        max_players: 8,
                        team_player_count: 4,
                    },
                    GameModeConfig {
                        name: "MTRanked".to_owned(),
                        short_name: "mtr".to_owned(),
                        matchmaker_type: MatchmakerConfig::MultiThreadedCutLists,
                        match_functions_type: MatchFunctionsConfig::Mmr {
                            max_mmr_distance: 300,
                            waiting_time_weight: 10,
                        },
                        min_players: 2,
                        max_players: 8,
                        team_player_count: 4,
                    },
                ],
                supported_regions: vec![GameRegion {
                    region_system_name: "eu-central-1".to_owned(),
                    region_prefix: "eu".to_owned(),
                    region_endpoint: "http://ec2.eu-central-1.amazonaws.com/".to_owned(),
                }],
                reserved_player_session_timeout: 30,
            }),
        }
    }

    pub fn get_matchmaking_settings(&self) -> &MatchmakingSettings {
        &self.matchmaking_settings
    }

    pub fn get_supported_regions(&self) -> &[GameRegion] {
        &self.get_matchmaking_settings().supported_regions
    }

    pub fn is_region_supported(&self, region_system_name: &str) -> bool {
        self.get_matchmaking_settings()
            .supported_regions
            .iter()
            .any(|r| r.region_system_name == region_system_name)
    }
}

impl Default for MatchmakingSettingsDAL {
    fn default() -> Self {
        Self::new()
    }
}
