use crate::{
    Error, MatchmakingAssembler,
    MatchmakingWaitingTimeCache,
    NotificationCache, ItemCache, 
    GameServerManager, matchmaker::{Matchmaker, new_matchmaker, MatchmakerContext}, 
    QueueMap,
};
use cotonou_common::{
    matchmaking::{
        GameServerDAL,
        GameServer, GameServerHostType, GameServerId,
        MatchmakingCommand,
        MatchmakingCommandDAL,
        MatchmakingTicket, MatchmakingPlayerStatus, MatchmakingPlayer,
        MatchmakingFailedNotification, MatchmakingFailureReason, 
        MatchmakingSessionStatus, MatchmakingSession, SessionId, 
        MatchmakingCompletedNotification, 
        MatchmakingActivateSessionNotification, 
        MatchmakingServersFullNotification, 
        MatchmakingWaitingTimeDAL, 
        MatchmakingSessionDAL, 
        MatchmakingSettingsDAL, 
        MatchmakingTicketDAL
    },
    notifications::NotificationManager,
    unix_now,
    profile::ProfileId,
};
use std::{time::Duration, collections::HashMap};
use tokio::time::Instant;

const LOOP_DURATION: Duration = Duration::from_secs(1);

pub type TicketCache = ItemCache<ProfileId, MatchmakingTicket, MatchmakingTicketDAL>;
pub type SessionCache = ItemCache<SessionId, MatchmakingSession, MatchmakingSessionDAL>;

pub struct MatchmakingJob {
    region_system_name: String,
    _region_prefix: String,
    matchmaking_command_dal: MatchmakingCommandDAL,
    matchmaking_settings_dal: MatchmakingSettingsDAL,
    waiting_time_cache: MatchmakingWaitingTimeCache,
    matchmaking_assembler: MatchmakingAssembler,
    notification_cache: NotificationCache,
    servers: GameServerManager,
    tickets: TicketCache,
    sessions: SessionCache,
    created_sessions: QueueMap<SessionId>,
    matchmakers: HashMap<String, Box<dyn Matchmaker>>,
    matched_players: HashMap<ProfileId, SessionId>,
    activating_players: QueueMap<(ProfileId, SessionId)>,
    shutdown_receiver: tokio::sync::watch::Receiver<()>,
    can_create_new_sessions: bool,
}

impl MatchmakingJob {
    //-------------------------------------------------------------------------------------------------
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::too_many_arguments))]
    pub fn new(
        region_system_name: &str,
        region_prefix: &str,
        game_server_dal: GameServerDAL,
        matchmaking_command_dal: MatchmakingCommandDAL,
        matchmaking_session_dal: MatchmakingSessionDAL,
        matchmaking_ticket_dal: MatchmakingTicketDAL,
        matchmaking_waiting_time_dal: MatchmakingWaitingTimeDAL,
        matchmaking_assembler: MatchmakingAssembler,
        notification_manager: NotificationManager,
        matchmaking_settings_dal: MatchmakingSettingsDAL,
        shutdown_receiver: tokio::sync::watch::Receiver<()>,
    ) -> Self {
        Self {
            region_system_name: region_system_name.to_owned(),
            _region_prefix: region_prefix.to_owned(),
            matchmaking_command_dal,
            matchmaking_settings_dal: matchmaking_settings_dal.clone(),
            waiting_time_cache: MatchmakingWaitingTimeCache::new(
                region_system_name,
                matchmaking_waiting_time_dal,
            ),
            matchmaking_assembler,
            notification_cache: NotificationCache::new(notification_manager),
            shutdown_receiver,
            can_create_new_sessions: true,
            servers: GameServerManager::new(region_system_name, game_server_dal),
            tickets: ItemCache::new(region_system_name, matchmaking_ticket_dal),
            sessions: ItemCache::new(region_system_name, matchmaking_session_dal),
            created_sessions: QueueMap::new(),
            matchmakers: matchmaking_settings_dal
                .get_matchmaking_settings()
                .game_mode_configs
                .iter()
                .map(|c|
                    (
                        c.name.clone(), 
                        new_matchmaker(region_system_name, c.clone())
                    ))
                .collect(),
            matched_players: HashMap::new(),
            activating_players: QueueMap::new(),
        }
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn job_loop(&mut self) -> Result<(), Error> {
        log::info!("[{}] MatchmakingJob started", self.region_system_name);

        if let Err(e) = self.load_cache().await {
            log::error!(
                "[{}] Error while loading cache: {:?}",
                self.region_system_name,
                e
            );
            return Err(e);
        }

        while !self.shutdown_receiver.has_changed()? {
            let start = Instant::now();

            self.process_commands().await?;
            self.process_servers();
            self.process_matchmakers();
            self.process_sessions();
            self.process_players();

            if let Err(e) = self.save_cache().await {
                log::error!(
                    "[{}] Error while saving cache: {:?}",
                    self.region_system_name,
                    e
                );
                return Err(e);
            }

            let elapsed = start.elapsed();
            if elapsed < LOOP_DURATION {
                tokio::time::sleep(LOOP_DURATION - elapsed).await;
            }
        }

        log::info!("MatchmakingJob loop stopped");

        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    async fn load_cache(&mut self) -> Result<(), Error> {
        let results = tokio::join!(
            self.servers.load(),
            self.tickets.load(),
            self.sessions.load()
        );

        results.0?;
        results.1?;
        results.2?;

        for ticket in self.tickets.iter() {
            let Some(matchmaker) = self.matchmakers.get_mut(&ticket.game_mode) else {
                log::error!("[{}] Cannot find matchmaker for game mode {}", self.region_system_name, ticket.game_mode);
                continue;
            };

            // ticket to match
            if ticket.session_id.is_none() {
                matchmaker.insert_ticket(ticket);
            }
        }

        for session in self.sessions.iter() {
            let Some(matchmaker) = self.matchmakers.get_mut(&session.game_mode) else {
                log::error!("[{}] Cannot find matchmaker for game mode {}", self.region_system_name, session.game_mode);
                continue;
            };

            // session to match
            if session.is_open {
                matchmaker.insert_session(session);
            }
        }

        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    async fn save_cache(&mut self) -> Result<(), Error> {
        let results = tokio::join!(
            self.servers.save(),
            self.tickets.save(),
            self.sessions.save(),
            self.waiting_time_cache.save_cache(),
            self.notification_cache.send_notifications()
        );

        results.0?;
        results.1?;
        results.2?;
        results.3?;
        results.4?;

        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    async fn process_commands(&mut self) -> Result<(), Error> {
        let commands = self
            .matchmaking_command_dal
            .dequeue_commands(&self.region_system_name)
            .await?;

        for command in commands {
            self.process_command(command).await?;
        }

        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    async fn process_command(&mut self, command: MatchmakingCommand) -> Result<(), Error> {
        match command {
            MatchmakingCommand::ActivatePlayerSession {
                session_id,
                profile_id,
            } => {
                self.activate_player_session(session_id, profile_id);
            }
            MatchmakingCommand::ActivateSession { session_id } => self.activate_session(session_id),
            MatchmakingCommand::CreateTicket { ticket } => self.create_ticket(ticket),
            MatchmakingCommand::DeleteSession { session_id } => self.command_delete_session(session_id),
            MatchmakingCommand::DeletePlayerSession {
                session_id,
                profile_id,
            } => self.delete_player_session(session_id, profile_id),
            MatchmakingCommand::DeleteTicket {
                owner_profile_id,
                player_who_canceled_profile_id,
            } => {
                self.delete_ticket(owner_profile_id, player_who_canceled_profile_id);
            }
            MatchmakingCommand::InitializeGameServer {
                game_server_id,
                host_name,
                host_type,
                host_boot_time,
                host_provider,
                game_version,
                process_id,
                ip_address,
                port,
            } => self.initialize_game_server(
                game_server_id,
                host_name,
                host_type,
                host_boot_time,
                host_provider,
                game_version,
                process_id,
                ip_address,
                port,
            ),
            MatchmakingCommand::KeepAliveGameServer { game_server_id } => {
                self.keep_alive_game_server(game_server_id)
            }
            MatchmakingCommand::ResetMatchmaking => self.reset_matchmaking().await?,
            MatchmakingCommand::ShutdownGameServer { game_server_id } => {
                self.shutdown_server(game_server_id)
            }
            MatchmakingCommand::UpdateSession {
                session_id,
                is_open,
            } => self.update_session(session_id, is_open),
        }

        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    fn activate_player_session(&mut self, session_id: SessionId, profile_id: ProfileId) {
        let Some(session) = self.sessions.get_mut(&session_id) else {
            log::error!("[{}] Cannot find session {session_id} while activating player {profile_id}", self.region_system_name);
            return;
        };

        let Some(player) = session.players.iter_mut().find(|p| p.profile_id == profile_id) else {
            log::error!("[{}] Cannot find player to activate {profile_id} in session {session_id}", self.region_system_name);
            return;
        };

        player.new_status_time = unix_now();
        player.status = MatchmakingPlayerStatus::Active;

        log::trace!("[{}] Player activated {profile_id} in session {session_id}", self.region_system_name);       

        self.sessions.update(session_id);
        self.tickets.delete(&profile_id);     
    }

    //-------------------------------------------------------------------------------------------------
    fn activate_session(&mut self, session_id: SessionId) {
        let Some(session) = self.sessions.get_mut(&session_id) else {
            log::error!("[{}] Cannot find session to activate {session_id}", self.region_system_name);
            return;
        };

        if session.status == MatchmakingSessionStatus::Active {
            log::warn!("[{}] Session {session_id} is already active", self.region_system_name);
            return;
        }

        log::trace!("[{}] Session {session_id} activated on server {:?}", self.region_system_name, session.game_server_id);     

        session.status = MatchmakingSessionStatus::Active;
        self.sessions.update(session_id);      
    }

    //-------------------------------------------------------------------------------------------------
    fn create_ticket(&mut self, ticket: MatchmakingTicket) {
        if self
            .tickets
            .delete(&ticket.owner_profile_id)
            .is_some()
        {
            log::warn!(
                "[{}] Ticket for player {} already existed and was replaced.",
                self.region_system_name,
                ticket.owner_profile_id
            );
        }
        
        log::trace!(
            "[{}] Ticket created for players {} with requested game mode {}.",
            self.region_system_name,
            ticket
                .players
                .iter()
                .map(|p| p.profile_id.to_string())
                .collect::<Vec<String>>()
                .join(","),
            ticket.game_mode
        );

        let Some(matchmaker) = self.matchmakers.get_mut(&ticket.game_mode) else {
            log::error!("[{}] Cannot find matchmaker for game mode {}", self.region_system_name, ticket.game_mode);
            return;
        };

        matchmaker.insert_ticket(&ticket);
        self.tickets.create(ticket);
    }

    //-------------------------------------------------------------------------------------------------
    fn command_delete_session(&mut self, session_id: SessionId) {
        let Some(session) = self.sessions.get(&session_id) else {
            log::error!("[{}] Cannot find session to delete {session_id}", self.region_system_name);
            return;
        };

        if let Some(server_id) = session.game_server_id {
            if let Some(server) = self.servers.get_server_mut(&server_id) {
                server.session_id = None;
                let game_server_id = server.game_server_id;
                self.servers.update_server(game_server_id);
            } else {
                log::warn!("[{}] Cannot find server {} while deleting session {session_id}", self.region_system_name, server_id);
            };
        }

        self.delete_session(&session_id);
    }

    //-------------------------------------------------------------------------------------------------
    fn delete_player_session(&mut self, session_id: SessionId, profile_id: ProfileId) {
        let Some(session) = self.sessions.get_mut(&session_id) else {
            log::error!("[{}] Cannot find session {session_id} while deleting player {profile_id}", self.region_system_name);
            return;
        };

        if self.tickets.delete(&profile_id).is_some() {
            log::trace!("[{}] Ticket deleted for player {profile_id}", self.region_system_name);
        }

        let old_num_players = session.players.len();
        session.players.retain(|p| p.profile_id != profile_id);
        let new_num_players = session.players.len();
        if new_num_players != old_num_players {
            log::error!("[{}] Cannot find player to delete {profile_id} in session {session_id}", self.region_system_name);
            return;
        }

        log::trace!("[{}] Player deleted {profile_id} from session {session_id}. Session now contains {new_num_players} player(s)", self.region_system_name);
        self.sessions.update(session_id);
    }

    //-------------------------------------------------------------------------------------------------
    fn delete_ticket(
        &mut self,
        owner_profile_id: ProfileId,
        player_who_canceled_profile_id: ProfileId,
    ) {
        let Some(ticket) = self.tickets.get(&owner_profile_id) else {
            log::warn!("[{}] Cannot find ticket to delete for player {owner_profile_id}", self.region_system_name);
            return;
        };

        let Some(matchmaker) = self.matchmakers.get_mut(&ticket.game_mode) else {
            log::error!("[{}] Cannot find matchmaker for game mode {}", self.region_system_name, ticket.game_mode);
            return;
        };

        if let Some(session_id) = ticket.session_id {
            if let Some(session) = self.sessions.get_mut(&session_id) {
                let players_to_delete = session.players
                    .iter()
                    .filter_map(|p| if ticket.players.iter().any(|tp| tp.profile_id == p.profile_id) {
                        Some(p.profile_id) 
                    } else {
                        None
                    })
                    .collect::<Vec<_>>();
                session.players.retain(|p| !players_to_delete.contains(&p.profile_id));
                log::trace!("[{}] Players deleted {} from session {}. Session now contains {} player(s)", 
                    self.region_system_name,
                    players_to_delete
                        .iter()
                        .map(|p| p.to_string())
                        .collect::<Vec<String>>()
                        .join(","),
                    session_id,
                    session.players.len()
                );
            }
        }

        for player in ticket.players.iter().filter(|p| p.profile_id != player_who_canceled_profile_id) {
            self.notification_cache.queue_player_notification(player.profile_id, MatchmakingFailedNotification {
                onwer_profile_id: ticket.owner_profile_id,
                failure_reason: MatchmakingFailureReason::CancelledByFriend,
            });
        }

        log::trace!("[{}] Ticket deleted for players {}", 
            self.region_system_name,
            ticket.players
                .iter()
                .map(|p| p.profile_id.to_string())
                .collect::<Vec<String>>()
                .join(","),
        );

        matchmaker.remove_ticket(ticket);
        self.tickets.delete(&owner_profile_id);
    }

    //-------------------------------------------------------------------------------------------------
    #[allow(clippy::too_many_arguments)]
    fn initialize_game_server(
        &mut self,
        game_server_id: GameServerId,
        host_name: String,
        host_type: GameServerHostType,
        host_boot_time: u64,
        host_provider: String,
        game_version: String,
        process_id: u32,
        ip_address: String,
        port: u16,
    ) {
        if self.servers.get_server(&game_server_id).is_some() {
            log::trace!(
                "[{}] Game server {} already initialized",
                self.region_system_name,
                game_server_id
            );
            return;
        }

        let server = GameServer {
            game_server_id,
            host_name,
            host_type,
            host_boot_time,
            host_provider,
            game_version,
            process_id,
            ip_address,
            port,
            session_id: None,
            keep_alive_time: unix_now(),
        };

        log::trace!("[{}] Game server id={}, ip_address={}, port={}, host_name={}, host_boot_time={}, host_provider={}, host_type={}, process_id={}, game_version={} initialized",
            self.region_system_name, 
            server.game_server_id, 
            server.ip_address, 
            server.port, 
            server.host_name, 
            server.host_boot_time, 
            server.host_provider,
            server.host_type, 
            server.process_id, 
            server.game_version);

        self.servers.create_server(server);
    }

    //-------------------------------------------------------------------------------------------------
    fn keep_alive_game_server(&mut self, game_server_id: GameServerId) {
        self.servers.keep_alive_server(game_server_id);
    }

    //-------------------------------------------------------------------------------------------------
    async fn reset_matchmaking(&mut self) -> Result<(), Error>  {
        // Cancel tickets to match
        for ticket in self.tickets.iter().filter(|t| t.session_id.is_none()) {
            for player in &ticket.players {
                self.notification_cache.queue_player_notification(player.profile_id, MatchmakingFailedNotification {
                    onwer_profile_id: ticket.owner_profile_id,
                    failure_reason: MatchmakingFailureReason::CancelledByMatchmakingService,
                })
            }
        }

        self.reset_cache().await
    }

    //-------------------------------------------------------------------------------------------------
    async fn reset_cache(&mut self) -> Result<(), Error> 
    {
        // Reset caches. Caches will reset the data they own in Redis
        let results = tokio::join!(
            self.servers.reset(),
            self.tickets.reset(),
            self.sessions.reset(),
            self.waiting_time_cache.reset(),
        );

        results.0?;
        results.1?;
        results.2?;
        results.3?;

        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    fn shutdown_server(&mut self, game_server_id: GameServerId) {
        let Some(server) = self.servers.get_server(&game_server_id) else {
            log::warn!("[{}] Cannot find game server {game_server_id} to shutdown", self.region_system_name);
            return;
        };

        if let Some(session_id) = server.session_id {
            self.delete_session(&session_id)
        }

        self.servers.delete_server(&game_server_id);

        log::trace!("[{}] Game server {game_server_id} shut down", self.region_system_name);
    }

    //-------------------------------------------------------------------------------------------------
    fn update_session(&mut self, session_id: SessionId, is_open: bool) {
        let Some(session) = self.sessions.get_mut(&session_id) else {
            log::error!("[{}] Cannot find session to update {session_id}", self.region_system_name);
            return;
        };

        if session.status != MatchmakingSessionStatus::Active {
            log::warn!("[{}] Cannot update session {session_id} because it is not active (status={:?})", self.region_system_name, session.status);
            return;
        }

        if session.is_open && !is_open {
            let now = unix_now();

            for player in &mut session.players {
                player.time_until_close_session = now - player.creation_time;
            }
        }

        session.is_open = is_open;

        if is_open
        {
            log::trace!("[{}] Session {session_id} is now open", self.region_system_name);
        }
        else
        {
            log::trace!("[{}] Session {session_id} is now closed", self.region_system_name);
        }

        if !session.is_open {
            let Some(matchmaker) = self.matchmakers.get_mut(&session.game_mode) else {
                log::error!("[{}] Cannot find matchmaker for game mode {}", self.region_system_name, session.game_mode);
                return;
            };

            matchmaker.remove_session(session);
        }
        self.sessions.update(session_id);
    }

    //-------------------------------------------------------------------------------------------------
    fn process_players(&mut self) {
        self.process_matched_players();
        self.process_activating_players();
    }

    //-------------------------------------------------------------------------------------------------
    fn process_matched_players(&mut self) {
        let now = unix_now();

        for (profile_id, session_id) in &self.matched_players {
            let Some(session) = self.sessions.get_mut(session_id) else {
                log::error!("[{}] Cannot find session {session_id} for matched player {profile_id}", self.region_system_name);
                continue;
            };

            let Some(player) = session.players.iter_mut().find(|p| p.profile_id == *profile_id) else {
                log::error!("[{}] Cannot find matched player {profile_id} in session {session_id}", self.region_system_name);
                continue;
            };

            player.new_status_time = now;
            player.status = MatchmakingPlayerStatus::Activating;
            Self::update_waiting_times(player, &self.tickets, &mut self.waiting_time_cache);

            self.notification_cache.queue_player_notification(player.profile_id, MatchmakingCompletedNotification {
                matchmaking_session_id: session.session_id.to_owned(),
                game_mode: session.game_mode.clone(),
                ip_address: session.ip_address.to_owned(),
                port: session.port,
                encryption_key: session.encryption_key.to_owned(),
            });

            self.sessions.update(session_id.to_owned());
            self.activating_players.insert((profile_id.to_owned(), session_id.to_owned()));
        }

        self.matched_players.clear();
    }

    //-------------------------------------------------------------------------------------------------
    fn process_activating_players(&mut self) {
        let now = unix_now();
        let timeout = self.matchmaking_settings_dal.get_matchmaking_settings().reserved_player_session_timeout;

        let players_to_delete = self.activating_players
            .iter()
            .take_while(|(profile_id, session_id)| {
                let Some(session) = self.sessions.get_mut(session_id) else {
                    log::error!("[{}] Cannot find session {session_id} for matched player {profile_id}", self.region_system_name);
                    return false;
                };
    
                let Some(player) = session.players
                    .iter_mut()
                    .find(|p| p.profile_id == *profile_id) else {
                    log::error!("[{}] Cannot find matched player {profile_id} in session {session_id}", self.region_system_name);
                    return false;
                };

                now - player.new_status_time > timeout
            })
            .map(|(profile_id, session_id)| (profile_id.to_owned(), session_id.to_owned()))
            .collect::<Vec<_>>();

        for (profile_id, session_id) in players_to_delete {
            let Some(session) = self.sessions.get_mut(&session_id) else {
                log::error!("[{}] Cannot find session {session_id} for matched player {profile_id}", self.region_system_name);
                continue;
            };

            log::trace!("[{}] Player {} timed out in session {}", self.region_system_name, profile_id, session.session_id);

            self.tickets.delete(&profile_id);

            session.players.retain(|p| p.profile_id != profile_id);
            self.sessions.update(session_id);
            self.activating_players.remove(&(profile_id, session_id));
        }
    }

    //-------------------------------------------------------------------------------------------------
    fn update_waiting_times(
        player: &mut MatchmakingPlayer, 
        tickets: &TicketCache, 
        waiting_time_cache: &mut MatchmakingWaitingTimeCache
    ) {
        let Some(ticket) = tickets.get(&player.profile_id) else {
            return;
        };

        // no estimated waiting time for friend parties
        if ticket.players.len() != 1 {
            return;
        }

        waiting_time_cache.update_cache(ticket, player);
    }

    //-------------------------------------------------------------------------------------------------
    fn delete_session(&mut self, session_id: &SessionId) {
        let Some(session) = self.sessions.get_mut(session_id) else {
            log::error!("[{}] Cannot find session to delete {session_id}", self.region_system_name);
            return;
        };

        for player in &session.players {
            let Some(ticket) = self.tickets.get_mut(&player.profile_id) else {
                continue;
            };

            match player.status {
                MatchmakingPlayerStatus::Created => (),
                MatchmakingPlayerStatus::Matched => {
                    ticket.session_id = None;
                    let owner_profile_id = ticket.owner_profile_id;
                    let Some(matchmaker) = self.matchmakers.get_mut(&ticket.game_mode) else {
                        log::error!("[{}] Cannot find matchmaker for game mode {}", self.region_system_name, ticket.game_mode);
                        return;
                    };
                    matchmaker.insert_ticket(ticket);
                    self.tickets.update(owner_profile_id);
                    log::trace!("[{}] Ticket reset for player {}", self.region_system_name, player.profile_id);
                },
                MatchmakingPlayerStatus::Activating | MatchmakingPlayerStatus::Active => {
                    self.tickets.delete(&player.profile_id);
                    log::trace!("[{}] Ticket deleted for player {}", self.region_system_name, player.profile_id);
                },
            }
        }

        let Some(matchmaker) = self.matchmakers.get_mut(&session.game_mode) else {
            log::error!("[{}] Cannot find matchmaker for game mode {}", self.region_system_name, session.game_mode);
            return;
        };

        if session.is_open {
            matchmaker.remove_session(session);
        }
        self.sessions.delete(session_id);
    }

    //-------------------------------------------------------------------------------------------------
    fn process_servers(&mut self) {
        let expired_session_ids = self.servers.process_expired_servers();
        
        for session_id in expired_session_ids {
            self.delete_session(&session_id);
        }

        self.can_create_new_sessions = self.servers.has_idle_server();

        // Notify all remaining tickets that server are full
        if !self.can_create_new_sessions {
            let now = unix_now();

            for (position_in_queue, owner_profile_id) in self.tickets
                .iter()
                .filter_map(|t| if t.session_id.is_none() { Some(t.owner_profile_id) } else { None })
                .enumerate()
                .collect::<Vec<_>>() {

                let Some(ticket) = self.tickets.get_mut(&owner_profile_id) else {
                    continue;
                };

                //send notifications each 10 seconds
                if now <= ticket.servers_full_notification_last_time_sent + 10 {
                    continue;
                }

                for player in ticket.players.iter() {
                    self.notification_cache.queue_player_notification(
                        player.profile_id,
                        MatchmakingServersFullNotification {
                            position_in_queue: position_in_queue + 1,
                            estimated_wait_time: self
                                .waiting_time_cache
                                .get_average_waiting_time(&ticket.game_mode),
                        },
                    );
                }

                ticket.servers_full_notification_last_time_sent = now;
                self.tickets.update(owner_profile_id);
            }
        }
    }

    //-------------------------------------------------------------------------------------------------
    fn process_matchmakers(&mut self) {
        if !self.can_create_new_sessions {
            return;
        }

        for matchmaker in self.matchmakers.values_mut() {
            matchmaker.process(
                &mut MatchmakerContext::new(
                    &self.region_system_name,
                    &mut self.tickets,
                    &mut self.sessions,
                    &mut self.created_sessions,
                    &mut self.matched_players,
                    &mut self.matchmaking_assembler,
                ));
        }
    }

    //-------------------------------------------------------------------------------------------------
    fn process_sessions(&mut self) {
        if !self.can_create_new_sessions {
            return;
        }

        let mut sessions_to_delete = Vec::new();

        for session_id in self.created_sessions.iter() {
            if let Some(game_server) = self.servers.get_idle_server_mut() {
                let Some(session) = self.sessions.get_mut(session_id) else {
                    log::error!("[{}] Cannot find created session {session_id}", self.region_system_name);
                    continue;
                };

                let Some(game_config) = self.matchmaking_settings_dal
                    .get_matchmaking_settings()
                    .game_mode_configs
                    .iter()
                    .find(|config| config.name == session.game_mode) else {
                    log::error!("[{}] Cannot find game mode config for game mode {}", self.region_system_name, session.game_mode);
                    continue;
                };

                let server_id = game_server.game_server_id;

                self.notification_cache.queue_gamer_server_notification(server_id, 
                    MatchmakingActivateSessionNotification {
                        matchmaking_session_id: session.session_id,
                        game_mode: session.game_mode.clone(),
                        encryption_key: session.encryption_key.clone(),
                        max_players: game_config.max_players,
                        team_player_count: game_config.team_player_count,
                    });

                game_server.session_id = Some(session.session_id);
                self.servers.update_server(server_id);

                session.game_server_id = Some(server_id);
                self.sessions.update(*session_id);
            } else {
                sessions_to_delete.push(*session_id);
            }
        }

        self.created_sessions.clear();

        for session_id in sessions_to_delete {
            // delete new sessions without a server
            // tickets will be put back into matchmakers
            self.delete_session(&session_id);
        }
    }
}

