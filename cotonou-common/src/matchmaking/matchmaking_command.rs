use crate::{
    matchmaking::{game_server::GameServerHostType, matchmaking_ticket::MatchmakingTicket},
    types::GameServerId,
    types::ProfileId,
};
use serde::{Deserialize, Serialize};

use super::matchmaking_session::SessionId;

#[derive(Serialize, Deserialize)]
pub enum MatchmakingCommand {
    ActivatePlayerSession {
        session_id: SessionId,
        profile_id: ProfileId,
    },
    ActivateSession {
        session_id: SessionId,
    },
    CreateTicket {
        ticket: MatchmakingTicket,
    },
    DeleteSession {
        session_id: SessionId,
    },
    DeletePlayerSession {
        session_id: SessionId,
        profile_id: ProfileId,
    },
    DeleteTicket {
        owner_profile_id: ProfileId,
        player_who_canceled_profile_id: ProfileId,
    },
    InitializeGameServer {
        game_server_id: GameServerId,
        host_name: String,
        host_type: GameServerHostType,
        host_boot_time: u64,
        host_provider: String,
        game_version: String,
        process_id: u32,
        ip_address: String,
        port: u16,
    },
    KeepAliveGameServer {
        game_server_id: GameServerId,
    },
    ResetMatchmaking,
    ShutdownGameServer {
        game_server_id: GameServerId,
    },
    UpdateSession {
        session_id: SessionId,
        is_open: bool,
    },
}
