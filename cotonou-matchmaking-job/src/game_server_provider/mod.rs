mod static_game_server_provider;
mod idle_server_collection;
mod auto_scaling_game_server_provider;
mod dynamic_host_provider;

use cotonou_common::matchmaking::game_server::{GameServer, GameServerId};

pub trait GameServerProvider {
    fn get_idle_server_mut(&mut self) -> Option<&mut GameServer>;
    fn get_server(&self, server_id: &GameServerId) -> Option<&GameServer>;
    fn process();
}
