use super::{idle_server_collection::IdleServerCollection, dynamic_host_provider::{DynamicHostProvider}};
use crate::queue_map::QueueMap;
use cotonou_common::{
    matchmaking::game_server::{GameServer, GameServerHostType, GameServerId, HostName},
    unix_now,
};
use std::collections::{hash_map::Entry, HashMap, HashSet};

const BUSY_RATIO_SCALE_UP_THRESHOLD: f64 = 0.7;
const BUSY_RATIO_SCALE_DOWN_THRESHOLD: f64 = 0.7;
const MIN_GAME_SERVERS: usize = 1;
const MIN_IDLE_GAME_SERVERS: usize = 1;
const TARGET_BUSY_RATIO: f64 = 0.7;
const AUTO_SCALING_SCALE_DOWN_COOLDOWN_SECONDS: u64 = 60;
const NUM_SERVERS_PER_HOST: usize = 4;

pub struct GameServerHost {
    pub host_name: HostName,
    pub host_provider: String,
    pub host_type: GameServerHostType,
    pub idle_servers: HashSet<GameServerId>,
    pub busy_servers: HashSet<GameServerId>,
    pub num_starting_servers: usize,
    pub host_boot_request_time: u64,
}

#[derive(PartialEq, PartialOrd)]
pub struct HostKey {
    pub host_type: GameServerHostType,
    pub occupation_ratio: f64,
    pub boot_time: u64,
    pub host_name: HostName,
}

pub struct AutoScalingGameServerProvider {
    dynamic_host_provider: DynamicHostProvider,
    last_auto_scaling_time: u64,
    num_idle_servers: usize,
    num_busy_servers: usize,
    num_starting_servers: usize,
    num_stopping_servers: usize,
    starting_hosts: HashMap<HostName, usize>,
    starting_host_queue: QueueMap<HostName>, // for expiration management
    stopping_hosts: HashMap<HostName, GameServerHost>,
}

impl AutoScalingGameServerProvider {
    pub fn new(dynamic_host_provider: DynamicHostProvider) -> Self {
        Self {
            dynamic_host_provider,
            last_auto_scaling_time: 0,
            num_idle_servers: 0,
            num_busy_servers: 0,
            num_starting_servers: 0,
            num_stopping_servers: 0,
            starting_hosts: HashMap::new(),
            starting_host_queue: QueueMap::new(),
            stopping_hosts: HashMap::new(),
        }
    }

    pub fn insert_server(&mut self, server: &GameServer) {
        if let Entry::Occupied(mut entry) = self.starting_hosts.entry(server.host_name.clone()) {
            let value = entry.get_mut();
            if *value == 1 {
                entry.remove_entry();
                self.starting_host_queue.remove(&server.host_name);
            } else {
                *value -= 1;
            }
            self.num_starting_servers -= 1;
        }

        if server.is_stopping() {
            self.num_stopping_servers += 1;
        } else if server.is_busy() {
            self.num_busy_servers += 1;
        } else {
            self.num_idle_servers += 1;
        }
    }

    pub fn set_server_busy(&mut self, server: &GameServer) {
        self.num_idle_servers -= 1;
        self.num_busy_servers += 1;
    }

    pub fn remove_server(&mut self, server: &GameServer) {
        if server.is_stopping() {
            self.num_stopping_servers -= 1;
        } else if server.is_busy() {
            self.num_busy_servers -= 1;
        } else {
            self.num_idle_servers -= 1;
        }
    }

    pub async fn process(&mut self, idle_servers: &IdleServerCollection) {
        self.process_hosts();
        self.auto_scale().await;
    }

    fn process_hosts(&mut self) {
        let now = unix_now();

        self.starting_host_queue.iter().take_while(|h| true);
    }

    async fn auto_scale(&mut self) {
        let num_servers = self.num_busy_servers + self.num_idle_servers + self.num_starting_servers;
        let busy_ratio = self.num_busy_servers as f64 / f64::max(1., num_servers as f64);

        if busy_ratio <= BUSY_RATIO_SCALE_UP_THRESHOLD
            && busy_ratio >= BUSY_RATIO_SCALE_DOWN_THRESHOLD
            && num_servers >= MIN_GAME_SERVERS
            && self.num_idle_servers + self.num_starting_servers >= MIN_IDLE_GAME_SERVERS
        {
            return;
        }

        let target_num_servers =
            f64::ceil(num_servers as f64 * (busy_ratio / TARGET_BUSY_RATIO)) as usize;
        let target_num_servers = usize::max(target_num_servers, MIN_GAME_SERVERS);
        let target_num_servers = usize::max(
            target_num_servers,
            self.num_busy_servers
                + usize::max(
                    self.num_idle_servers + self.num_starting_servers,
                    MIN_IDLE_GAME_SERVERS,
                ),
        );
        let delta_num_servers = target_num_servers as i32 - num_servers as i32;

        if delta_num_servers > 0 {
            self.scale_up(delta_num_servers as usize).await;
        } else if delta_num_servers < 0 && !self.is_scale_down_cooldown_active() {
            self.scale_down(-delta_num_servers as usize).await;
        }
    }

    async fn scale_up(&self, num_game_servers: usize) {
        let starting_hosts = self.dynamic_host_provider.start_game_servers(num_game_servers).await;
        self.num_starting_servers += starting_hosts.iter().map(|h| h.num_game_servers).sum();
        // set as starting
    }

    async fn scale_down(&self, num_game_servers: usize) {
        if num_game_servers < NUM_SERVERS_PER_HOST {
            return;
        }

        let stoppable_hosts = self.get_stoppable_hosts();
        let hosts_to_stop = stoppable_hosts.take(num_game_servers).cloned().collect::<Vec<_>>();
        self.dynamic_host_provider.stop_game_servers(&hosts_to_stop).await;

        for host_name in hosts_to_stop {
            // set as stopping
        }
    }

    fn is_scale_down_cooldown_active(&self) -> bool {
        self.last_auto_scaling_time + AUTO_SCALING_SCALE_DOWN_COOLDOWN_SECONDS > unix_now()
    }

    fn get_stoppable_hosts<'a>(&self) -> impl Iterator<Item = &'a HostName> {
        todo!()
    }
}
