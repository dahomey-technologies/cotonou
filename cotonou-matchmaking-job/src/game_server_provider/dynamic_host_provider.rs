use cotonou_common::matchmaking::game_server::HostName;

pub struct DynamicHostProvider {

}

pub struct StartingHost {
    pub host_name: HostName,
    pub num_game_servers: usize,
}

impl DynamicHostProvider {
    pub async fn start_game_servers(&self, num_game_servers: usize) -> Vec<StartingHost> {
        todo!()
    }

    pub async fn stop_game_servers(&self, hosts_to_stop: &[HostName]) {
        todo!()
    }
}