use crate::{Error, MatchmakingAssembler, MatchmakingJob};
use common_macros::hash_map;
use cotonou_common::{
    matchmaking::{
        GameServerDAL, MatchmakingCommandDAL, MatchmakingSessionDAL, MatchmakingSettingsDAL,
        MatchmakingTicketDAL, MatchmakingWaitingTimeDAL,
    },
    notifications::NotificationManager,
    redis::{RedisConfig, RedisConnectionManager},
};
use tokio::task::JoinSet;

pub struct MatchmakingMasterJob {
    matchmaking_settings_dal: MatchmakingSettingsDAL,
    game_server_dal: GameServerDAL,
    matchmaking_command_dal: MatchmakingCommandDAL,
    matchmaking_session_dal: MatchmakingSessionDAL,
    matchmaking_ticket_dal: MatchmakingTicketDAL,
    matchmaking_waiting_time_dal: MatchmakingWaitingTimeDAL,
    matchmaking_assembler: MatchmakingAssembler,
    notification_manager: NotificationManager,
    shutdown_receiver: tokio::sync::watch::Receiver<()>,
}

impl MatchmakingMasterJob {
    pub async fn new(shutdown_receiver: tokio::sync::watch::Receiver<()>) -> Result<Self, Error> {
        let redis_host = "127.0.0.1";
        let redis_connection_manager = RedisConnectionManager::initialize(RedisConfig {
            connection_strings: hash_map! {
                "NOTIFICATIONS".to_owned() => format!("redis://{redis_host}:6379/0"),
                "NOTIFICATIONS_PUBSUB".to_owned() => format!("redis://{redis_host}:6379/0"),
                "MATCHMAKING".to_owned() => format!("redis://{redis_host}:6379/1"),
            },
        })
        .await?;

        let game_server_dal = GameServerDAL::new(&redis_connection_manager);
        let matchmaking_command_dal = MatchmakingCommandDAL::new(&redis_connection_manager);
        let matchmaking_settings_dal = MatchmakingSettingsDAL::new();
        let matchmaking_session_dal = MatchmakingSessionDAL::new(&redis_connection_manager);
        let matchmaking_ticket_dal = MatchmakingTicketDAL::new(&redis_connection_manager);
        let matchmaking_waiting_time_dal =
            MatchmakingWaitingTimeDAL::new(&redis_connection_manager);
        let matchmaking_assembler = MatchmakingAssembler::new();
        let notification_manager = NotificationManager::new(&redis_connection_manager);

        Ok(Self {
            matchmaking_settings_dal,
            game_server_dal,
            matchmaking_command_dal,
            matchmaking_session_dal,
            matchmaking_ticket_dal,
            matchmaking_waiting_time_dal,
            matchmaking_assembler,
            notification_manager,
            shutdown_receiver,
        })
    }

    pub async fn initialize(&self) -> Result<(), Error> {
        let mut set = JoinSet::new();

        for region in self.matchmaking_settings_dal.get_supported_regions() {
            let mut matchmaking_job = MatchmakingJob::new(
                &region.region_system_name,
                &region.region_prefix,
                self.game_server_dal.clone(),
                self.matchmaking_command_dal.clone(),
                self.matchmaking_session_dal.clone(),
                self.matchmaking_ticket_dal.clone(),
                self.matchmaking_waiting_time_dal.clone(),
                self.matchmaking_assembler.clone(),
                self.notification_manager.clone(),
                self.matchmaking_settings_dal.clone(),
                self.shutdown_receiver.clone(),
            );

            set.spawn(async move { matchmaking_job.job_loop().await });
        }

        while let Some(_result) = set.join_next().await {}

        Ok(())
    }
}
