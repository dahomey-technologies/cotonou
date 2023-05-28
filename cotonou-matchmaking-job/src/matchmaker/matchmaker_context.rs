use crate::{
    matchmaking_assembler::MatchmakingAssembler,
    matchmaking_job::{SessionCache, TicketCache},
    queue_map::QueueMap,
};
use cotonou_common::{
    matchmaking::{
        matchmaking_session::{MatchmakingSession, MatchmakingSessionStatus, SessionId},
        matchmaking_ticket::MatchmakingTicket,
    },
    models::ProfileId,
    unix_now,
};
use std::collections::HashMap;

pub struct MatchmakerContext<'a> {
    region_system_name: &'a str,
    tickets: &'a mut TicketCache,
    sessions: &'a mut SessionCache,
    created_sessions: &'a mut QueueMap<SessionId>,
    matched_players: &'a mut HashMap<ProfileId, SessionId>,
    matchmaking_assembler: &'a mut MatchmakingAssembler,
}

impl<'a> MatchmakerContext<'a> {
    pub fn new(
        region_system_name: &'a str,
        tickets: &'a mut TicketCache,
        sessions: &'a mut SessionCache,
        created_sessions: &'a mut QueueMap<SessionId>,
        matched_players: &'a mut HashMap<ProfileId, SessionId>,
        matchmaking_assembler: &'a mut MatchmakingAssembler,
    ) -> Self {
        Self {
            region_system_name,
            tickets,
            sessions,
            created_sessions,
            matched_players,
            matchmaking_assembler,
        }
    }

    pub fn get_ticket(&self, owner_profile_id: &ProfileId) -> Option<&MatchmakingTicket> {
        self.tickets.get(owner_profile_id)
    }

    pub fn get_session(&self, session_id: &SessionId) -> Option<&MatchmakingSession> {
        self.sessions.get(session_id)
    }

    pub fn match_ticket_to_existing_session(
        &mut self,
        ticket_id: ProfileId,
        session_id: SessionId,
    ) {
        let Some(ticket) = self.tickets.get_mut(&ticket_id) else {
            return;
        };

        let Some(session) = self.sessions.get_mut(&session_id) else {
            return;
        };

        Self::match_ticket_to_session(
            self.region_system_name,
            self.matched_players,
            self.matchmaking_assembler,
            ticket,
            session,
        );

        self.tickets.update(ticket_id);
        self.sessions.update(session_id);
    }

    pub fn match_tickets_to_new_session(&mut self, session_id: SessionId, tickets_to_match: &[ProfileId]) {
        let Some(ticket) = self.tickets.get(&tickets_to_match[0]) else {
            unreachable!()
        };

        let mut session = MatchmakingSession {
            session_id,
            game_mode: ticket.game_mode.clone(),
            players: Vec::new(),
            creation_time: unix_now(),
            status: MatchmakingSessionStatus::Created,
            is_open: true,
            game_server_id: None,
            ip_address: String::from(""),
            port: 0,
            encryption_key: String::from(""),
        };

        let session_id = session.session_id;

        log::trace!(
            "[{}] Session {} created for game mode {}",
            self.region_system_name,
            session_id,
            ticket.game_mode
        );

        for ticket_id in tickets_to_match {
            if let Some(ticket) = self.tickets.get_mut(ticket_id) {
                Self::match_ticket_to_session(
                    self.region_system_name,
                    self.matched_players,
                    self.matchmaking_assembler,
                    ticket,
                    &mut session,
                );
                self.tickets.update(*ticket_id);
            }
        }

        self.sessions.create(session);
        self.created_sessions.insert(session_id);
    }

    fn match_ticket_to_session(
        region_system_name: &str,
        matched_players: &mut HashMap<ProfileId, SessionId>,
        matchmaking_assembler: &mut MatchmakingAssembler,
        ticket: &mut MatchmakingTicket,
        session: &mut MatchmakingSession,
    ) {
        let players_to_add = ticket
            .players
            .iter()
            .map(|p| matchmaking_assembler.convert_to_matchmaking_player(p, ticket.creation_time))
            .collect::<Vec<_>>();

        log::trace!(
            "[{}] Player(s) {} added to session {}. Session now contains {} player(s)",
            region_system_name,
            players_to_add
                .iter()
                .map(|p| p.profile_id.to_string())
                .collect::<Vec<String>>()
                .join(","),
            session.session_id,
            session.players.len()
        );

        session.players.extend(players_to_add);
        ticket.session_id = Some(session.session_id);

        matched_players.insert(ticket.owner_profile_id, session.session_id);
    }
}
