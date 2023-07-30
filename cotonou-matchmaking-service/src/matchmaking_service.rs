#[cfg(debug_assertions)]
use crate::AppState;
use crate::{
    Error, MatchmakingAssembler,
    MatchmakingStartedNotification,
    ProfileForMatchmakingManager,
};
use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use cotonou_common::{
    authentication::{JwtRole, User},
    matchmaking::{
        MatchmakingCommand, MatchmakingCommandDAL, MatchmakingPlayerStatus, MatchmakingSessionDAL,
        MatchmakingSettingsDAL, MatchmakingTicketDAL, MatchmakingWaitingTimeDAL, SessionId,
    },
    notifications::NotificationManager,
    types::ProfileId,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize)]
pub struct CreateMatchmakingTicketResponse {
    pub estimated_wait_time: u64,
}

#[derive(Deserialize)]
pub struct MatchmakingTicketPlayer {
    pub profile_id: ProfileId,
    pub latency: u32,
}

#[derive(Deserialize)]
pub struct CreateMatchmakingTicketRequest {
    pub game_mode: String,
    pub players: Vec<MatchmakingTicketPlayer>,
    pub client_version: String,
}

/// Create a matchmaking ticket (client only)
#[allow(clippy::too_many_arguments)]
#[axum::debug_handler(state = AppState)]
pub async fn create_matchmaking_ticket(
    State(matchmaking_settings_dal): State<Arc<MatchmakingSettingsDAL>>,
    State(profile_for_matchmaking_manager): State<Arc<ProfileForMatchmakingManager>>,
    State(matchmaking_assembler): State<Arc<MatchmakingAssembler>>,
    State(matchmaking_command_dal): State<Arc<MatchmakingCommandDAL>>,
    State(notification_manager): State<Arc<NotificationManager>>,
    State(matchmaking_waiting_time_dal): State<Arc<MatchmakingWaitingTimeDAL>>,
    Extension(user): Extension<User>,
    Path((region_system_name, owner_profile_id)): Path<(String, ProfileId)>,
    Json(request): Json<CreateMatchmakingTicketRequest>,
) -> Result<Json<CreateMatchmakingTicketResponse>, Error> {
    if !matches!(user.role, JwtRole::Player) {
        return Err(Error::Unauthorized);
    }

    if owner_profile_id != user.get_profile_id() {
        return Err(Error::Unauthorized);
    }

    validate_region(&matchmaking_settings_dal, &region_system_name)?;

    // TODO check if we are in maintenance mode (server are closed)

    if !request
        .players
        .iter()
        .any(|p| p.profile_id == owner_profile_id)
    {
        return Err(Error::InvalidParameter("body.players".to_owned()));
    }

    let Some(game_mode_config) = matchmaking_settings_dal
        .get_matchmaking_settings()
        .game_mode_configs
        .iter()
        .find(|gmc| gmc.name == request.game_mode) else {
        return Err(Error::InvalidParameter("body.game_mode".to_owned()));
    };

    // check max players
    if request.players.len() > game_mode_config.max_players {
        return Err(Error::InvalidParameter("body.players.len".to_owned()));
    }

    // TODO check client version

    let players_profile_ids = request
        .players
        .iter()
        .map(|p| p.profile_id)
        .collect::<Vec<_>>();

    let profiles_for_matchmaking = profile_for_matchmaking_manager
        .get_profiles_for_matchmaking(&players_profile_ids)
        .await?;

    if profiles_for_matchmaking.len() != players_profile_ids.len() {
        let unknown_online_ids = players_profile_ids
            .iter()
            .filter(|id| profiles_for_matchmaking.iter().all(|p| p.id != **id))
            .collect::<Vec<_>>();

        log::warn!("Cannot find profile for onlineId(s) {unknown_online_ids:?}");
        return Err(Error::InvalidParameter("body.players".to_owned()));
    }

    let ticket = matchmaking_assembler.convert_to_matchmaking_ticket(
        owner_profile_id,
        &request,
        &profiles_for_matchmaking,
    );

    let game_mode = ticket.game_mode.clone();
    let mut estimated_wait_time: u64 = 0;

    // estimated wait time when no friend party
    if ticket.players.len() == 1 {
        estimated_wait_time = matchmaking_waiting_time_dal
            .get_average_waiting_time(&region_system_name, &game_mode)
            .await?;
    }

    matchmaking_command_dal
        .queue_command(
            &region_system_name,
            &MatchmakingCommand::CreateTicket { ticket },
        )
        .await?;

    for player in request
        .players
        .iter()
        .filter(|p| p.profile_id != owner_profile_id)
    {
        notification_manager
            .send_notification(
                &player.profile_id.to_string(),
                &MatchmakingStartedNotification {
                    owner_profile_id,
                    region_system_name: region_system_name.clone(),
                    game_mode: game_mode.clone(),
                },
            )
            .await?;
    }

    Ok(Json(CreateMatchmakingTicketResponse {
        estimated_wait_time,
    }))
}

/// Delete a matchmaking ticket (client only)
pub async fn delete_matchmaking_ticket(
    State(matchmaking_settings_dal): State<Arc<MatchmakingSettingsDAL>>,
    State(matchmaking_ticket_dal): State<Arc<MatchmakingTicketDAL>>,
    State(matchmaking_command_dal): State<Arc<MatchmakingCommandDAL>>,
    Extension(user): Extension<User>,
    Path((region_system_name, owner_profile_id)): Path<(String, ProfileId)>,
) -> Result<(), Error> {
    let Some(ticket) = matchmaking_ticket_dal.get_ticket(&region_system_name, owner_profile_id).await? else {
        return Err(Error::InvalidParameter("owner_profile_id".to_owned()));
    };

    let current_profile_id = user.get_profile_id();
    if !ticket
        .players
        .iter()
        .any(|p| p.profile_id == current_profile_id)
    {
        return Err(Error::Unauthorized);
    };

    validate_region(&matchmaking_settings_dal, &region_system_name)?;

    matchmaking_command_dal
        .queue_command(
            &region_system_name,
            &MatchmakingCommand::DeleteTicket {
                owner_profile_id,
                player_who_canceled_profile_id: current_profile_id,
            },
        )
        .await?;

    Ok(())
}

/// Active a matchmaking session (server only)
pub async fn activate_session(
    State(matchmaking_settings_dal): State<Arc<MatchmakingSettingsDAL>>,
    State(matchmaking_command_dal): State<Arc<MatchmakingCommandDAL>>,
    Path((region_system_name, session_id)): Path<(String, SessionId)>,
) -> Result<(), Error> {
    validate_region(&matchmaking_settings_dal, &region_system_name)?;

    matchmaking_command_dal
        .queue_command(
            &region_system_name,
            &MatchmakingCommand::ActivateSession { session_id },
        )
        .await?;

    Ok(())
}

#[derive(Deserialize)]
pub struct UpdateSessionQuery {
    pub is_open: bool,
}

/// Update a matchmaking session (server only)
pub async fn update_session(
    State(matchmaking_settings_dal): State<Arc<MatchmakingSettingsDAL>>,
    State(matchmaking_command_dal): State<Arc<MatchmakingCommandDAL>>,
    Path((region_system_name, session_id)): Path<(String, SessionId)>,
    Query(query): Query<UpdateSessionQuery>,
) -> Result<(), Error> {
    validate_region(&matchmaking_settings_dal, &region_system_name)?;

    matchmaking_command_dal
        .queue_command(
            &region_system_name,
            &MatchmakingCommand::UpdateSession {
                session_id,
                is_open: query.is_open,
            },
        )
        .await?;

    Ok(())
}

/// Delete a matchmaking session (server only).
pub async fn delete_session(
    State(matchmaking_settings_dal): State<Arc<MatchmakingSettingsDAL>>,
    State(matchmaking_command_dal): State<Arc<MatchmakingCommandDAL>>,
    Path((region_system_name, session_id)): Path<(String, SessionId)>,
) -> Result<(), Error> {
    validate_region(&matchmaking_settings_dal, &region_system_name)?;

    matchmaking_command_dal
        .queue_command(
            &region_system_name,
            &MatchmakingCommand::DeleteSession { session_id },
        )
        .await?;

    Ok(())
}

/// Activate a matchmaking player session (server only).
/// Should be called when a client connects to a server to validate that the client is allowed by the matchmaking
pub async fn activate_player_session(
    State(matchmaking_settings_dal): State<Arc<MatchmakingSettingsDAL>>,
    State(matchmaking_session_dal): State<Arc<MatchmakingSessionDAL>>,
    State(matchmaking_command_dal): State<Arc<MatchmakingCommandDAL>>,
    Path((region_system_name, session_id, profile_id)): Path<(String, SessionId, ProfileId)>,
) -> Result<(), Error> {
    validate_region(&matchmaking_settings_dal, &region_system_name)?;

    let Some(session) = matchmaking_session_dal.get_session(&region_system_name, &session_id).await? else {
        return Err(Error::InvalidParameter("session_id".to_owned()));
    };

    let Some(player) = session.players.iter().find(|p| p.profile_id == profile_id) else {
        return Err(Error::InvalidParameter("profile_id".to_owned()));
    };

    if player.status != MatchmakingPlayerStatus::Activating {
        return Err(Error::InvalidParameter("profile_id".to_owned()));
    }

    matchmaking_command_dal
        .queue_command(
            &region_system_name,
            &MatchmakingCommand::ActivatePlayerSession {
                session_id,
                profile_id,
            },
        )
        .await?;

    Ok(())
}

/// Delete an existing matchmaking player session (server only).
pub async fn delete_player_session(
    State(matchmaking_settings_dal): State<Arc<MatchmakingSettingsDAL>>,
    State(matchmaking_command_dal): State<Arc<MatchmakingCommandDAL>>,
    Path((region_system_name, session_id, profile_id)): Path<(String, SessionId, ProfileId)>,
) -> Result<(), Error> {
    validate_region(&matchmaking_settings_dal, &region_system_name)?;

    matchmaking_command_dal
        .queue_command(
            &region_system_name,
            &MatchmakingCommand::DeletePlayerSession {
                session_id,
                profile_id,
            },
        )
        .await?;

    Ok(())
}

fn validate_region(
    matchmaking_settings_dal: &MatchmakingSettingsDAL,
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
