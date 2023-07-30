#[cfg(debug_assertions)]
use crate::app_state::AppState;
use crate::error::Error;
use axum::{
    extract::{Path, State},
    Extension, Json,
};
use cotonou_common::{
    game_server_dal::GameServerDAL,
    matchmaking::{
        game_server::{GameServerHostType, GameServerId},
        matchmaking_command::MatchmakingCommand,
        matchmaking_command_dal::MatchmakingCommandDAL,
    },
    matchmaking_settings_dal::MatchmakingSettingsDAL,
    user::User,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct InitializeGameServerRequest {
    pub host_provider: String,
    pub host_name: String,
    pub host_boot_time: u64,
    pub host_type: GameServerHostType,
    pub game_version: String,
    pub process_id: u32,
    pub ip_address: String,
    pub port: u16,
}

/// Called by a game server to register itself in the matchmaking (server only)
///
/// # Arguments
/// * `region_system_name` - e.g. us-east-1
/// * `game_server_id` - Unique id for this game server instance (uuid)
/// * `request` - Details of the game server registration
#[axum::debug_handler(state = AppState)]
pub async fn initialize_game_server(
    State(matchmaking_command_dal): State<Arc<MatchmakingCommandDAL>>,
    State(matchmaking_settings_dal): State<Arc<MatchmakingSettingsDAL>>,
    Extension(user): Extension<User>,
    Path((region_system_name, game_server_id)): Path<(String, GameServerId)>,
    Json(request): Json<InitializeGameServerRequest>,
) -> Result<(), Error> {
    validate_region(matchmaking_settings_dal, &region_system_name)?;

    log::info!("user: {user:?}");

    matchmaking_command_dal
        .queue_command(
            &region_system_name,
            &MatchmakingCommand::InitializeGameServer {
                game_server_id,
                host_name: request.host_name,
                host_type: request.host_type,
                host_boot_time: request.host_boot_time,
                host_provider: request.host_provider,
                game_version: request.game_version,
                process_id: request.process_id,
                ip_address: request.ip_address,
                port: request.port,
            },
        )
        .await?;
    Ok(())
}

/// Keep alive a game server in the matchmaking. Should be called every minute (server only)
///
/// # Arguments
/// * `region_system_name` - e.g. us-east-1
/// * `game_server_id` - id as passed to the game server registration
pub async fn keep_alive_game_server(
    State(matchmaking_settings_dal): State<Arc<MatchmakingSettingsDAL>>,
    State(matchmaking_command_dal): State<Arc<MatchmakingCommandDAL>>,
    State(game_server_dal): State<Arc<GameServerDAL>>,
    Path((region_system_name, game_server_id)): Path<(String, GameServerId)>,
) -> Result<(), Error> {
    validate_region(matchmaking_settings_dal, &region_system_name)?;

    let game_server = game_server_dal
        .get_game_server(&region_system_name, &game_server_id)
        .await?;

    if game_server.is_none() {
        return Err(Error::InvalidParameter("game_server_id".to_owned()));
    };

    matchmaking_command_dal
        .queue_command(
            &region_system_name,
            &MatchmakingCommand::KeepAliveGameServer { game_server_id },
        )
        .await?;
    Ok(())
}

/// Unregister a game server from the matchmaking
///
/// # Arguments
/// * `region_system_name` - e.g. us-east-1
/// * `game_server_id` - id as passed to the game server registration
pub async fn shutdown_game_server(
    State(matchmaking_settings_dal): State<Arc<MatchmakingSettingsDAL>>,
    State(matchmaking_command_dal): State<Arc<MatchmakingCommandDAL>>,
    State(game_server_dal): State<Arc<GameServerDAL>>,
    Path((region_system_name, game_server_id)): Path<(String, GameServerId)>,
) -> Result<(), Error> {
    validate_region(matchmaking_settings_dal, &region_system_name)?;

    let game_server = game_server_dal
        .get_game_server(&region_system_name, &game_server_id)
        .await?;

    if game_server.is_none() {
        return Err(Error::InvalidParameter("game_server_id".to_owned()));
    };

    matchmaking_command_dal
        .queue_command(
            &region_system_name,
            &MatchmakingCommand::ShutdownGameServer { game_server_id },
        )
        .await?;
    Ok(())
}

fn validate_region(
    matchmaking_settings_dal: Arc<MatchmakingSettingsDAL>,
    region_system_name: &str,
) -> Result<(), Error> {
    if !matchmaking_settings_dal.is_region_supported(region_system_name) {
        Err(Error::InvalidParameter(format!(
            "Matchmaking region [{region_system_name}"
        )))
    } else {
        Ok(())
    }
}
