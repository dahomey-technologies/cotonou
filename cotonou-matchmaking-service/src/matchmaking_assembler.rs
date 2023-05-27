use crate::{
    matchmaking_service::CreateMatchmakingTicketRequest,
    profile_for_matchmaking_entity::ProfileForMatchmakingEntity,
};
use cotonou_common::{
    matchmaking::matchmaking_ticket::{
        MatchmakingPlayer, MatchmakingPlayerStatus, MatchmakingTicket,
    },
    models::ProfileId,
    unix_now,
};

#[derive(Clone)]
pub struct MatchmakingAssembler;

impl MatchmakingAssembler {
    pub fn convert_to_matchmaking_ticket(
        &self,
        owner_online_id: ProfileId,
        request: &CreateMatchmakingTicketRequest,
        profiles_for_matchmaking: &[ProfileForMatchmakingEntity],
    ) -> MatchmakingTicket {
        MatchmakingTicket {
            owner_profile_id: owner_online_id,
            game_mode: request.game_mode.clone(),
            players: request
                .players
                .iter()
                .map(|p| {
                    let profile_for_matchmaking = profiles_for_matchmaking
                        .iter()
                        .find(|pfm| pfm.id == p.profile_id);

                    MatchmakingPlayer {
                        profile_id: p.profile_id,
                        display_name: profile_for_matchmaking.unwrap().display_name.clone(),
                        mmr: match profile_for_matchmaking {
                            Some(profile_for_matchmaking) => {
                                match profile_for_matchmaking.mmrs.get(&request.game_mode) {
                                    Some(elo) => *elo,
                                    None => 0u32,
                                }
                            }
                            None => 0u32,
                        },
                        latency: 0u32,
                        new_status_time: 0u64,
                        status: MatchmakingPlayerStatus::Created,
                        creation_time: 0u64,
                        time_until_open_session: 0u64,
                        time_until_close_session: 0u64,
                    }
                })
                .collect(),
            creation_time: unix_now(),
            session_id: None,
            servers_full_notification_last_time_sent: 0u64,
        }
    }
}
