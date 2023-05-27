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
};

pub struct FirstComeFirstServed {
    _region_system_name: String,
    game_mode_config: GameModeConfig,
    open_sessions: QueueMap<SessionId>,
    open_tickets: QueueMap<ProfileId>,
}

impl FirstComeFirstServed {
    pub fn new(region_system_name: &str, game_mode_config: GameModeConfig) -> Self {
        Self {
            _region_system_name: region_system_name.to_owned(),
            game_mode_config,
            open_sessions: QueueMap::new(),
            open_tickets: QueueMap::new(),
        }
    }
}

impl FirstComeFirstServed {
    //-------------------------------------------------------------------------------------------------
    fn is_ticket_with_session_match(
        &self,
        ticket: &MatchmakingTicket,
        session: &MatchmakingSession,
    ) -> bool {
        self.is_players_match(ticket.players.iter().chain(session.players.iter()))
    }

    //-------------------------------------------------------------------------------------------------
    fn is_ticket_with_ticket_match(
        &self,
        ticket1: &MatchmakingTicket,
        ticket2: &MatchmakingTicket,
    ) -> bool {
        self.is_players_match(ticket1.players.iter().chain(ticket2.players.iter()))
    }

    //-------------------------------------------------------------------------------------------------
    fn is_ticket_match(&self, ticket: &MatchmakingTicket) -> bool {
        self.is_players_match(ticket.players.iter())
    }

    //-------------------------------------------------------------------------------------------------
    fn is_players_match<'b, I: Iterator<Item = &'b MatchmakingPlayer>>(&self, players: I) -> bool {
        let num_players = players.count();
        num_players >= self.game_mode_config.min_players
            && num_players <= self.game_mode_config.max_players
    }

    //-------------------------------------------------------------------------------------------------
    fn process_until_session_creation(&mut self, context: &mut MatchmakerContext) -> bool {
        // search a match in existing sessions (join in progress)
        let mut matched_tickets = Vec::new();

        for ticket_id in self.open_tickets.iter() {
            let Some(ticket) = context.get_ticket(ticket_id) else {
                continue;
            };

            for session_id in self.open_sessions.iter() {
                let Some(session) = context.get_session(session_id) else {
                    continue;
                };

                if self.is_ticket_with_session_match(ticket, session) {
                    context.match_ticket_to_existing_session(*ticket_id, *session_id);
                    matched_tickets.push(*ticket_id);
                    break;
                }
            }
        }

        for ticket_id in matched_tickets {
            self.open_tickets.remove(&ticket_id);
        }

        // create new sessions
        for ticket1_id in self.open_tickets.iter() {
            for ticket2_id in self.open_tickets.iter() {
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

        false
    }
}

impl Matchmaker for FirstComeFirstServed {
    fn insert_ticket(&mut self, ticket: &MatchmakingTicket) {
        self.open_tickets.insert(ticket.owner_profile_id);
    }

    fn remove_ticket(&mut self, ticket: &MatchmakingTicket) {
        self.open_tickets.remove(&ticket.owner_profile_id);
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
