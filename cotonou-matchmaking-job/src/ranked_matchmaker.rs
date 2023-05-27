use std::iter::repeat;

use crate::{
    matchmaker::{Matchmaker, MatchmakerContext},
    queue_map::QueueMap,
};
use cotonou_common::{
    matchmaking::{
        matchmaking_session::{MatchmakingSession, SessionId},
        matchmaking_ticket::{MatchmakingPlayer, MatchmakingTicket},
    },
    models::{GameModeConfig, ProfileId},
    unix_now,
};

const TICKET_INITIAL_CAPACITY: usize = 10;
const MMR_RANGE: u32 = 100;
const MAX_MMR_DISTANCE: u32 = 300;
const WAITING_TIME_WEIGHT: u32 = 10;

/// Based cutlist algorithm described here:
/// "Scalable services for massively multiplayer online games" by Maxime Véron p.41
/// https://theses.hal.science/tel-01230852
pub struct RankedMatchmaker {
    _region_system_name: String,
    game_mode_config: GameModeConfig,
    open_sessions: QueueMap<SessionId>,
    open_tickets: Vec<QueueMap<ProfileId>>,
}

impl RankedMatchmaker {
    pub fn new(region_system_name: &str, game_mode_config: GameModeConfig) -> Self {
        Self {
            _region_system_name: region_system_name.to_owned(),
            game_mode_config,
            open_sessions: QueueMap::new(),
            open_tickets: Vec::with_capacity(TICKET_INITIAL_CAPACITY),
        }
    }

    fn get_average_mmr(&self, players: &[MatchmakingPlayer]) -> u32 {
        let sum = players.iter().fold(0, |acc, p| acc + p.mmr);
        sum / players.len() as u32
    }

    fn get_average_waiting_time(&self, players: &[MatchmakingPlayer]) -> u32 {
        let now = unix_now();
        let sum = players.iter().fold(0, |acc, p| acc + now - p.creation_time);
        sum as u32 / players.len() as u32
    }

    fn is_ticket_with_session_match(
        &self,
        ticket: &MatchmakingTicket,
        session: &MatchmakingSession,
    ) -> bool {
        if !self.is_size_player_match(ticket.players.iter().chain(session.players.iter())) {
            return false;
        }

        self.is_mmr_player_match(&ticket.players, &session.players)
    }

    fn is_ticket_with_ticket_match(
        &self,
        ticket1: &MatchmakingTicket,
        ticket2: &MatchmakingTicket,
    ) -> bool {
        if !self.is_size_player_match(ticket1.players.iter().chain(ticket2.players.iter())) {
            return false;
        }

        self.is_mmr_player_match(&ticket1.players, &ticket2.players)
    }

    fn is_ticket_match(&self, ticket: &MatchmakingTicket) -> bool {
        self.is_size_player_match(ticket.players.iter())
    }

    fn is_size_player_match<'b, I: Iterator<Item = &'b MatchmakingPlayer>>(
        &self,
        players: I,
    ) -> bool {
        let num_players = players.count();
        num_players >= self.game_mode_config.min_players
            && num_players <= self.game_mode_config.max_players
    }

    fn is_mmr_player_match(
        &self,
        group1: &[MatchmakingPlayer],
        group2: &[MatchmakingPlayer],
    ) -> bool {
        let distance = (self.get_average_mmr(group1) as i64 - self.get_average_mmr(group2) as i64)
            .unsigned_abs() as u32;
        let waiting_time = self.get_average_waiting_time(group1);

        u32::min(waiting_time * WAITING_TIME_WEIGHT, MAX_MMR_DISTANCE) > distance
    }

    fn process_until_session_creation(&mut self, context: &mut MatchmakerContext) -> bool {
        for i in 0..self.open_tickets.len() {
            let Some(open_tickets) = self.open_tickets.get(i) else {
                continue;
            };

            // search a match in existing sessions (join in progress)
            let mut matched_tickets = Vec::new();

            for ticket_id in open_tickets.iter() {
                let Some(ticket) = context.get_ticket(ticket_id) else {
                    continue;
                };

                for session_id in self.open_sessions.iter() {
                    let Some(session) = context.get_session(session_id) else {
                        continue;
                    };

                    if self.is_ticket_with_session_match(ticket, session) {
                        context.match_ticket_to_existing_session(*ticket_id, *session_id);
                        break;
                    }

                    matched_tickets.push(*ticket_id);
                }
            }

            let Some(open_tickets) = self.open_tickets.get_mut(i) else {
                continue;
            };

            for ticket_id in matched_tickets {
                open_tickets.remove(&ticket_id);
            }

            let Some(open_tickets) = self.open_tickets.get(i) else {
                continue;
            };

            // create new sessions
            for ticket1_id in open_tickets.iter() {
                for ticket2_id in open_tickets.iter() {
                    let Some(ticket1) = context.get_ticket(ticket1_id) else {
                        continue;
                    };

                    let Some(ticket2) = context.get_ticket(ticket2_id) else {
                        continue;
                    };

                    if ticket1.owner_profile_id != ticket2.owner_profile_id
                        && self.is_ticket_with_ticket_match(ticket1, ticket2)
                    {
                        let session_id =
                            context.match_tickets_to_new_session(&[*ticket1_id, *ticket2_id]);

                        self.open_sessions.insert(session_id);

                        // a new session has been created, restart processing tickets with existing sessions
                        return true;
                    }
                }

                let Some(ticket1) = context.get_ticket(ticket1_id) else {
                    continue;
                };

                if self.is_ticket_match(ticket1) {
                    let session_id = context.match_tickets_to_new_session(&[*ticket1_id]);

                    self.open_sessions.insert(session_id);
    
                    // a new session has been created, restart processing tickets with existing sessions
                    return true;
                }
            }
        }

        false
    }
}

impl Matchmaker for RankedMatchmaker {
    fn insert_ticket(&mut self, ticket: &MatchmakingTicket) {
        let mmr_index = (self.get_average_mmr(&ticket.players) / MMR_RANGE) as usize;
        if mmr_index >= self.open_tickets.len() {
            let num_elements_to_add = mmr_index - self.open_tickets.len() + 1;
            self.open_tickets
                .extend(repeat(QueueMap::new()).take(num_elements_to_add));
        }
        self.open_tickets[mmr_index].insert(ticket.owner_profile_id);
    }

    fn remove_ticket(&mut self, ticket: &MatchmakingTicket) {
        let mmr_index = (self.get_average_mmr(&ticket.players) / MMR_RANGE) as usize;
        if mmr_index < self.open_tickets.len() {
            self.open_tickets[mmr_index].remove(&ticket.owner_profile_id);
        }
    }

    fn insert_session(&mut self, session: &MatchmakingSession) {
        self.open_sessions.insert(session.session_id);
    }

    fn remove_session(&mut self, session: &MatchmakingSession) {
        self.open_sessions.remove(&session.session_id);
    }

    fn process(&mut self, context: &mut MatchmakerContext) {
        // when a new session has been created, restart processing tickets with existing sessions
        while self.process_until_session_creation(context) {}
    }
}
