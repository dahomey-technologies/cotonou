use crate::{matchmaking::Error, redis::RedisConnectionManager};
use rustis::{
    client::Client,
    commands::{GenericCommands, StringCommands},
};

#[derive(Clone)]
pub struct MatchmakingWaitingTimeDAL {
    client: Client,
}

impl MatchmakingWaitingTimeDAL {
    //-------------------------------------------------------------------------------------------------
    pub fn new(redis_connection_manager: &RedisConnectionManager) -> Self {
        Self {
            client: redis_connection_manager.get_client("MATCHMAKING").unwrap(),
        }
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn set_average_waiting_time(
        &self,
        region_system_name: &str,
        game_mode: &str,
        average_waiting_time: u64,
    ) -> Result<(), Error> {
        let key = build_key(region_system_name, game_mode);
        self.client.set(key, average_waiting_time).await?;
        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn get_average_waiting_time(
        &self,
        region_system_name: &str,
        game_mode: &str,
    ) -> Result<u64, Error> {
        let key = build_key(region_system_name, game_mode);
        let average_waiting_time: Option<u64> = self.client.get(key).await?;
        Ok(average_waiting_time.unwrap_or_default())
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn reset(&self, region_system_name: &str, game_mode: &str) -> Result<(), Error> {
        let key = build_key(region_system_name, game_mode);
        self.client.del(key).await?;
        Ok(())
    }
}

//-------------------------------------------------------------------------------------------------
fn build_key(region_system_name: &str, game_mode: &str) -> String {
    format!("{region_system_name}:{game_mode}:mmawt")
}
