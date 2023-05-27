use crate::{
    error::Error, matchmaking_assembler::MatchmakingAssembler,
    profile_for_matchmaking_manager::ProfileForMatchmakingManager,
};
use axum::extract::FromRef;
use common_macros::hash_map;
use cotonou_common::{
    generic_dal::GenericDAL,
    matchmaking::matchmaking_command_dal::MatchmakingCommandDAL,
    matchmaking_settings_dal::MatchmakingSettingsDAL,
    mongo::mongo_config::MongoConfig,
    notifications::notification_manager::NotificationManager,
    redis::{redis_config::RedisConfig, redis_connection_manager::RedisConnectionManager}, game_server_dal::GameServerDAL, matchmaking_ticket_dal::MatchmakingTicketDAL, matchmaking_session_dal::MatchmakingSessionDAL, matchmaking_average_waiting_time_dal::MatchmakingWaitingTimeDAL,
};
use std::sync::Arc;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub profile_for_matchmaking_manager: Arc<ProfileForMatchmakingManager>,
    pub matchmaking_assembler: Arc<MatchmakingAssembler>,
    pub matchmaking_command_dal: Arc<MatchmakingCommandDAL>,
    pub notification_manager: Arc<NotificationManager>,
    pub matchmaking_settings_dal: Arc<MatchmakingSettingsDAL>,
    pub game_server_dal: Arc<GameServerDAL>,
    pub matchmaking_ticket_dal: Arc<MatchmakingTicketDAL>,
    pub matchmaking_session_dal: Arc<MatchmakingSessionDAL>,
    pub matchmaking_waiting_time_dal: Arc<MatchmakingWaitingTimeDAL>,
}

impl AppState {
    pub async fn new() -> Result<AppState, Error> {
        let mongo_host = "mongo";
        let redis_host = "redis";

        let generic_dal = GenericDAL::initialize(&MongoConfig {
            connection_string: format!("mongodb://{mongo_host}:27017/test"),
        })
        .await?;

        let redis_connection_manager = RedisConnectionManager::initialize(RedisConfig {
            connection_strings: hash_map! {
                "NOTIFICATIONS".to_owned() => format!("redis://{redis_host}:6379/0"),
                "NOTIFICATIONS_PUBSUB".to_owned() => format!("redis://{redis_host}:6379/0"),
                "MATCHMAKING".to_owned() => format!("redis://{redis_host}:6379/1"),
            },
        })
        .await
        .unwrap();

        let profile_for_matchmaking_manager =
            Arc::new(ProfileForMatchmakingManager::new(generic_dal.clone()));
        let matchmaking_assembler = Arc::new(MatchmakingAssembler);
        let matchmaking_command_dal =
            Arc::new(MatchmakingCommandDAL::new(&redis_connection_manager));
        let notification_manager = Arc::new(NotificationManager::new(&redis_connection_manager));
        let matchmaking_settings_dal = Arc::new(MatchmakingSettingsDAL::new());
        let game_server_dal = Arc::new(GameServerDAL::new(&redis_connection_manager));
        let matchmaking_ticket_dal = Arc::new(MatchmakingTicketDAL::new(&redis_connection_manager));
        let matchmaking_session_dal = Arc::new(MatchmakingSessionDAL::new(&redis_connection_manager));
        let matchmaking_waiting_time_dal = Arc::new(MatchmakingWaitingTimeDAL::new(&redis_connection_manager));

        Ok(Self {
            profile_for_matchmaking_manager,
            matchmaking_assembler,
            matchmaking_command_dal,
            notification_manager,
            matchmaking_settings_dal,
            game_server_dal,
            matchmaking_ticket_dal,
            matchmaking_session_dal,
            matchmaking_waiting_time_dal
        })
    }
}
