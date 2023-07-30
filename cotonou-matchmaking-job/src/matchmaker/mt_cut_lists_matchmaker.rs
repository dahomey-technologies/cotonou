use crate::{
    match_functions::{is_in_bounds, MatchFunctions},
    matchmaker::{Matchmaker, MatchmakerContext},
    QueueMap,
    get_average_mmr,
};
use cotonou_common::{
    matchmaking::{
        GameModeConfig, MatchmakingPlayer, MatchmakingSession, MatchmakingTicket, SessionId,
    },
    types::ProfileId,
};
use std::{
    collections::HashMap,
    sync::mpsc::{self, TryRecvError},
    thread::{self, JoinHandle},
};

/// Based Multi-Threaded CutLists algorithm described here:
/// "Scalable services for massively multiplayer online games" by Maxime Véron p.42
/// https://theses.hal.science/tel-01230852
pub struct MultiThreadedCutListsMatchmaker {
    _region_system_name: String,
    game_mode_config: GameModeConfig,
    match_functions: Box<dyn MatchFunctions>,
    mmr_range: u32,
    jobs: Vec<JoinHandle<()>>,
    msg_senders: Vec<mpsc::Sender<MessageToJob>>,
    msg_receivers: Vec<mpsc::Receiver<MessageFromJob>>,
}

impl MultiThreadedCutListsMatchmaker {
    pub fn new(
        region_system_name: &str,
        game_mode_config: GameModeConfig,
        match_functions: Box<dyn MatchFunctions>,
        mmr_range: u32,
    ) -> Self {
        Self {
            _region_system_name: region_system_name.to_owned(),
            game_mode_config,
            match_functions,
            mmr_range,
            jobs: Vec::new(),
            msg_senders: Vec::new(),
            msg_receivers: Vec::new(),
        }
    }

    fn new_job(&mut self) {
        let (msg_to_job_sender, msg_to_job_receiver) = mpsc::channel();
        let (msg_from_job_sender, msg_from_job_receiver) = mpsc::channel();

        let mut job = Job::new(
            self.game_mode_config.clone(),
            self.match_functions.clone(),
            msg_from_job_sender,
            msg_to_job_receiver,
        );
        self.jobs.push(thread::spawn(move || job.job_loop()));
        self.msg_senders.push(msg_to_job_sender);
        self.msg_receivers.push(msg_from_job_receiver);
    }
}

impl Drop for MultiThreadedCutListsMatchmaker {
    fn drop(&mut self) {
        for sender in &mut self.msg_senders.iter() {
            if let Err(e) = sender.send(MessageToJob::Shutdown) {
                log::error!("Cannot shutdown job in MTCutListsMatchmaker: {e}");
                continue;
            }
        }

        while let Some(job) = self.jobs.pop() {
            if let Err(e) = job.join() {
                log::error!("Cannot shutdown job in MTCutListsMatchmaker: {e:?}");
                continue;
            }
        }

        log::info!("MTCutListsMatchmaker stopped");
    }
}

impl Matchmaker for MultiThreadedCutListsMatchmaker {
    fn insert_ticket(&mut self, ticket: &MatchmakingTicket) {
        let mmr_index = (get_average_mmr(&ticket.players) / self.mmr_range) as usize;
        if mmr_index >= self.jobs.len() {
            let num_elements_to_add = mmr_index - self.jobs.len() + 1;
            for _ in 0..num_elements_to_add {
                self.new_job();
            }
        }
        if let Err(e) = self.msg_senders[mmr_index].send(MessageToJob::InsertTicket {
            owner_profile_id: ticket.owner_profile_id,
            players: ticket.players.clone(),
        }) {
            log::error!("Cannot insert ticket into MTCutListsMatchmaker: {e}");
        }
    }

    fn remove_ticket(&mut self, ticket: &MatchmakingTicket) {
        let mmr_index = (get_average_mmr(&ticket.players) / self.mmr_range) as usize;
        if let Some(sender) = self.msg_senders.get(mmr_index) {
            if let Err(e) = sender.send(MessageToJob::RemoveTicket {
                owner_profile_id: ticket.owner_profile_id,
            }) {
                log::error!("Cannot remove ticket from MTCutListsMatchmaker: {e}");
            }
        } else {
            log::error!("Cannot remove ticket from MTCutListsMatchmaker");
        }
    }

    fn insert_session(&mut self, session: &MatchmakingSession) {
        let mmr_index = (get_average_mmr(&session.players) / self.mmr_range) as usize;
        if let Some(sender) = self.msg_senders.get(mmr_index) {
            if let Err(e) = sender.send(MessageToJob::InsertSession {
                session_id: session.session_id,
                players: session.players.clone(),
            }) {
                log::error!("Cannot insert session into MTCutListsMatchmaker: {e}");
            }
        } else {
            log::error!("Cannot insert session into MTCutListsMatchmaker");
        }
    }

    fn remove_session(&mut self, session: &MatchmakingSession) {
        let mmr_index = (get_average_mmr(&session.players) / self.mmr_range) as usize;
        if let Some(sender) = self.msg_senders.get(mmr_index) {
            if let Err(e) = sender.send(MessageToJob::RemoveSession {
                session_id: session.session_id,
            }) {
                log::error!("Cannot remove session from MTCutListsMatchmaker: {e}");
            }
        } else {
            log::error!("Cannot remove session from MTCutListsMatchmaker");
        }
    }

    fn process(&mut self, context: &mut MatchmakerContext) {
        for cut_list_idx in 0..self.msg_receivers.len() {
            let msg_receiver = &mut self.msg_receivers[cut_list_idx];

            while let Some(message) = match msg_receiver.try_recv() {
                Ok(message) => Some(message),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => {
                    log::info!("CutList job ended");
                    return;
                }
            } {
                match message {
                    MessageFromJob::MatchToExistingSession {
                        ticket_id,
                        session_id,
                    } => context.match_ticket_to_existing_session(ticket_id, session_id),
                    MessageFromJob::MatchToNewSession {
                        session_id,
                        tickets_to_match,
                    } => context.match_tickets_to_new_session(session_id, &tickets_to_match),
                }
            }
        }
    }
}

pub enum MessageToJob {
    InsertTicket {
        owner_profile_id: ProfileId,
        players: Vec<MatchmakingPlayer>,
    },
    RemoveTicket {
        owner_profile_id: ProfileId,
    },
    InsertSession {
        session_id: SessionId,
        players: Vec<MatchmakingPlayer>,
    },
    RemoveSession {
        session_id: SessionId,
    },
    Shutdown,
}

pub enum MessageFromJob {
    MatchToExistingSession {
        ticket_id: ProfileId,
        session_id: SessionId,
    },
    MatchToNewSession {
        session_id: SessionId,
        tickets_to_match: Vec<ProfileId>,
    },
}

pub struct Job {
    game_mode_config: GameModeConfig,
    match_functions: Box<dyn MatchFunctions>,
    msg_sender: mpsc::Sender<MessageFromJob>,
    msg_receiver: mpsc::Receiver<MessageToJob>,
    open_sessions: QueueMap<SessionId>,
    open_tickets: QueueMap<ProfileId>,
    session_players: HashMap<SessionId, Vec<MatchmakingPlayer>>,
    ticket_players: HashMap<ProfileId, Vec<MatchmakingPlayer>>,
}

impl Job {
    pub fn new(
        game_mode_config: GameModeConfig,
        match_functions: Box<dyn MatchFunctions>,
        msg_sender: mpsc::Sender<MessageFromJob>,
        msg_receiver: mpsc::Receiver<MessageToJob>,
    ) -> Self {
        Self {
            game_mode_config,
            match_functions,
            msg_sender,
            msg_receiver,
            open_sessions: QueueMap::new(),
            open_tickets: QueueMap::new(),
            session_players: HashMap::new(),
            ticket_players: HashMap::new(),
        }
    }

    pub fn job_loop(&mut self) {
        loop {
            while let Some(message) = match self.msg_receiver.try_recv() {
                Ok(message) => Some(message),
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => {
                    log::info!("CutList job ended");
                    return;
                }
            } {
                match message {
                    MessageToJob::InsertTicket {
                        owner_profile_id,
                        players,
                    } => {
                        self.open_tickets.insert(owner_profile_id);
                        self.ticket_players.insert(owner_profile_id, players);
                    }
                    MessageToJob::RemoveTicket { owner_profile_id } => {
                        self.open_tickets.remove(&owner_profile_id);
                        self.ticket_players.remove(&owner_profile_id);
                    }
                    MessageToJob::InsertSession {
                        session_id,
                        players,
                    } => {
                        self.open_sessions.insert(session_id);
                        self.session_players.insert(session_id, players);
                    }
                    MessageToJob::RemoveSession { session_id } => {
                        self.open_sessions.remove(&session_id);
                        self.session_players.remove(&session_id);
                    }
                    MessageToJob::Shutdown => {
                        log::info!("CutList job ended");
                        return;
                    }
                }
            }

            self.process();
        }
    }

    pub fn process(&mut self) {
        // when a new session has been created, restart processing tickets with existing sessions
        while self.process_until_session_creation() {}
    }

    fn process_until_session_creation(&mut self) -> bool {
        // search a match in existing sessions (join in progress)
        let mut matched_tickets = Vec::new();

        for ticket_id in self.open_tickets.iter() {
            let Some(ticket_players) = self.ticket_players.get(ticket_id) else {
                    continue;
                };

            for session_id in self.open_sessions.iter() {
                let Some(session_players) = self.session_players.get(session_id) else {
                        continue;
                    };

                if self.match_functions.is_match(
                    &self.game_mode_config,
                    ticket_players,
                    session_players,
                ) {
                    if let Err(e) = self
                        .msg_sender
                        .send(MessageFromJob::MatchToExistingSession {
                            ticket_id: *ticket_id,
                            session_id: *session_id,
                        })
                    {
                        log::error!("Cannot send MessageFromJob::MatchToExistingSession: {e}");
                    }
                    break;
                }

                matched_tickets.push(*ticket_id);
            }
        }

        for ticket_id in matched_tickets {
            self.open_tickets.remove(&ticket_id);
            self.ticket_players.remove(&ticket_id);
        }

        // create new sessions
        for ticket1_id in self.open_tickets.iter() {
            for ticket2_id in self.open_tickets.iter().filter(|id| **id != *ticket1_id) {
                let Some(ticket1_players) = self.ticket_players.get(ticket1_id) else {
                    continue;
                 };

                let Some(ticket2_players) = self.ticket_players.get(ticket2_id) else {
                    continue;
                };

                if self.match_functions.is_match(
                    &self.game_mode_config,
                    ticket1_players,
                    ticket2_players,
                ) {
                    let session_id = SessionId::new();
                    if let Err(e) = self.msg_sender.send(MessageFromJob::MatchToNewSession {
                        session_id,
                        tickets_to_match: vec![*ticket1_id, *ticket2_id],
                    }) {
                        log::error!("Cannot send MessageFromJob::MatchToNewSession: {e}");
                    }

                    self.open_sessions.insert(session_id);
                    self.session_players.insert(
                        session_id,
                        ticket1_players
                            .iter()
                            .chain(ticket2_players.iter())
                            .cloned()
                            .collect(),
                    );

                    // a new session has been created, restart processing tickets with existing sessions
                    return true;
                }
            }

            let Some(ticket1_players) = self.ticket_players.get(ticket1_id) else {
                continue;
             };

            if is_in_bounds(&self.game_mode_config, ticket1_players.iter()) {
                let session_id = SessionId::new();

                if let Err(e) = self.msg_sender.send(MessageFromJob::MatchToNewSession {
                    session_id,
                    tickets_to_match: vec![*ticket1_id],
                }) {
                    log::error!("Cannot send MessageFromJob::MatchToNewSession: {e}");
                }

                self.open_sessions.insert(session_id);
                self.session_players
                    .insert(session_id, ticket1_players.clone());

                // a new session has been created, restart processing tickets with existing sessions
                return true;
            }
        }

        false
    }
}
