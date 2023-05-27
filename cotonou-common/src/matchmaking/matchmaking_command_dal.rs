use rustis::{client::Client, commands::ListCommands};

use super::{matchmaking_command::MatchmakingCommand, redis_key_names};
use crate::redis::redis_connection_manager::RedisConnectionManager;
use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Json,
    Redis,
}

impl From<serde_json::Error> for Error {
    fn from(json_error: serde_json::Error) -> Self {
        println!("Json Error: {:?}", json_error);
        Error::Json
    }
}

impl From<rustis::Error> for Error {
    fn from(_: rustis::Error) -> Self {
        Error::Redis
    }
}

#[derive(Clone)]
pub struct MatchmakingCommandDAL {
    client: Client,
}

impl MatchmakingCommandDAL {
    pub fn new(redis_connection_manager: &RedisConnectionManager) -> Self {
        Self {
            client: redis_connection_manager.get_client("MATCHMAKING").unwrap(),
        }
    }

    pub async fn queue_command(
        &self,
        region_system_name: &str,
        command: &MatchmakingCommand,
    ) -> Result<()> {
        let key = self.build_queue_key_name(region_system_name);
        let json = serde_json::to_string(command)?;
        self.client.rpush(key, json).await?;
        Ok(())
    }

    pub async fn dequeue_commands(
        &self,
        region_system_name: &str,
    ) -> Result<Vec<MatchmakingCommand>> {
        let key = self.build_queue_key_name(region_system_name);
        Ok(self
            .client
            .lpop::<_, _, Vec<String>>(key, 1000)
            .await?
            .iter()
            .map(|c| serde_json::from_str::<MatchmakingCommand>(c).unwrap())
            .collect())
    }

    fn build_queue_key_name(&self, region_system_name: &str) -> String {
        format!("{}:{}", region_system_name, redis_key_names::COMMAND_QUEUE)
    }
}
