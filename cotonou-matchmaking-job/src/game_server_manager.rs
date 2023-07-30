use crate::{Error, ItemCache, QueueMap};
use cotonou_common::{
    matchmaking::{GameServer, GameServerDAL, SessionId},
    types::GameServerId,
    unix_now,
};
use std::{collections::HashSet, time::Duration};

pub type ServerQueueMap = QueueMap<GameServerId>;
pub type ServerCache = ItemCache<GameServerId, GameServer, GameServerDAL>;

const GAME_SERVER_TIMEOUT: Duration = Duration::from_secs(60);

pub struct GameServerManager {
    region_system_name: String,
    servers: ServerCache,
    active_servers: ServerQueueMap,
    idle_servers: HashSet<GameServerId>,
}

impl GameServerManager {
    pub fn new(region_system_name: &str, game_server_dal: GameServerDAL) -> Self {
        Self {
            region_system_name: region_system_name.to_owned(),
            servers: ServerCache::new(region_system_name, game_server_dal),
            active_servers: ServerQueueMap::new(),
            idle_servers: HashSet::new(),
        }
    }

    pub async fn load(&mut self) -> Result<(), Error> {
        self.servers.load().await?;

        for server in self.servers.iter() {
            self.active_servers.insert(server.game_server_id);
            if server.session_id.is_none() {
                self.idle_servers.insert(server.game_server_id);
            }
        }

        Ok(())
    }

    pub async fn save(&mut self) -> Result<(), Error> {
        self.servers.save().await
    }

    pub async fn reset(&mut self) -> Result<(), Error> {
        self.active_servers.clear();
        self.idle_servers.clear();
        self.servers.reset().await
    }

    pub fn process_expired_servers(&mut self) -> Vec<SessionId> {
        let keep_alive_timeout = GAME_SERVER_TIMEOUT.as_secs();

        let now = unix_now();

        let expired_servers = self
            .active_servers
            .iter()
            .filter_map(|id| {
                if let Some(server) = self.servers.get(id) {
                    Some((*id, server.keep_alive_time, server.session_id))
                } else {
                    log::error!("[{}] Cannot find server {id}", self.region_system_name);
                    None
                }
            })
            .take_while(|(_, keep_alive_time, _)| keep_alive_time + keep_alive_timeout <= now)
            .collect::<Vec<_>>();

        for (server_id, keep_alive_time, _) in &expired_servers {
            log::warn!(
                "[{}] Game server {} expired with last keep alive time={}",
                self.region_system_name,
                server_id,
                keep_alive_time,
            );
            self.delete_server(server_id);
        }

        expired_servers
            .into_iter()
            .filter_map(|(_, _, session_id)| session_id)
            .collect()
    }

    pub fn has_idle_server(&self) -> bool {
        !self.idle_servers.is_empty()
    }

    pub fn get_idle_server_mut(&mut self) -> Option<&mut GameServer> {
        self.idle_servers
            .iter()
            .next()
            .and_then(|id| self.servers.get_mut(id))
    }

    pub fn get_server(&self, server_id: &GameServerId) -> Option<&GameServer> {
        self.servers.get(server_id)
    }

    pub fn get_server_mut(&mut self, server_id: &GameServerId) -> Option<&mut GameServer> {
        self.servers.get_mut(server_id)
    }

    pub fn create_server(&mut self, game_server: GameServer) -> bool {
        let server_id = game_server.game_server_id;
        let session_id = game_server.session_id;

        if !self.servers.create(game_server) {
            return false;
        }
        self.active_servers.insert(server_id);
        if session_id.is_none() {
            self.idle_servers.insert(server_id);
        }
        true
    }

    pub fn update_server(&mut self, server_id: GameServerId) {
        let Some(server) = self.servers.get(&server_id) else {
            log::error!("[{}] Cannot find game server {} to update", self.region_system_name, server_id);
            return;
        };
        if server.session_id.is_none() {
            self.idle_servers.insert(server_id);
        } else {
            self.idle_servers.remove(&server_id);
        }
        self.servers.update(server_id);
    }

    pub fn delete_server(&mut self, server_id: &GameServerId) -> Option<GameServer> {
        self.active_servers.remove(server_id);
        self.idle_servers.remove(server_id);
        self.servers.delete(server_id)
    }

    pub fn keep_alive_server(&mut self, server_id: GameServerId) {
        let Some(server) = self.servers.get_mut(&server_id) else {
            log::error!("[{}] Cannot find game server {} to keep alive", self.region_system_name, server_id);
            return;
        };

        server.keep_alive_time = unix_now();
        self.active_servers.remove(&server_id);
        self.active_servers.insert(server_id);
        self.servers.update(server_id);

        //log::trace!("[{}] Game server {server_id} kept alive", self.region_system_name);
    }
}
