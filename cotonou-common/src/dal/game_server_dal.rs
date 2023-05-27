use crate::{
    matchmaking::game_server::{GameServer, GameServerId}, redis::redis_connection_manager::RedisConnectionManager,
};
use rustis::{
    client::Client,
    commands::{GenericCommands, SetCommands, StringCommands},
};

pub enum Error {
    Redis(rustis::Error),
    Json(serde_json::Error),
}

impl From<rustis::Error> for Error {
    fn from(error: rustis::Error) -> Self {
        Error::Redis(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::Json(error)
    }
}

#[derive(Clone)]
pub struct GameServerDAL {
    client: Client,
}

impl GameServerDAL {
    //-------------------------------------------------------------------------------------------------
    pub fn new(redis_connection_manager: &RedisConnectionManager) -> Self {
        Self {
            client: redis_connection_manager.get_client("MATCHMAKING").unwrap(),
        }
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn get_game_servers(
        &self,
        region_system_name: &str,
    ) -> Result<Vec<GameServer>, Error> {
        let game_server_ids: Vec<GameServerId> = self
            .client
            .smembers(build_game_server_set_key(region_system_name))
            .await?;

        if game_server_ids.is_empty() {
            return Ok(Vec::new());
        }

        let keys = game_server_ids
            .into_iter()
            .map(|id| build_game_server_key(region_system_name, &id))
            .collect::<Vec<_>>();
        let values: Vec<String> = self.client.mget(keys).await?;

        let servers = values
            .into_iter()
            .map(|v| serde_json::from_str(&v))
            .collect::<serde_json::Result<Vec<GameServer>>>()?;

        Ok(servers)
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn get_game_server(
        &self,
        region_system_name: &str,
        game_server_id: &GameServerId,
    ) -> Result<Option<GameServer>, Error> {
        let key = build_game_server_key(region_system_name, game_server_id);
        let session_json: Option<String> = self.client.get(key).await?;

        if let Some(session_json) = session_json {
            Ok(Some(serde_json::from_str(&session_json)?))
        } else {
            Ok(None)
        }
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn create_game_servers<'a, I: ExactSizeIterator<Item = &'a GameServer> + Clone>(
        &self,
        region_system_name: &str,
        game_servers: I,
    ) -> Result<(), Error> {
        if game_servers.len() == 0 {
            return Ok(());
        }

        let game_server_ids = game_servers
            .clone()
            .map(|gs| &gs.game_server_id)
            .copied()
            .collect::<Vec<_>>();
        self.client
            .sadd(
                build_game_server_set_key(region_system_name),
                game_server_ids,
            )
            .await?;

        let items = game_servers
            .map(|gs| {
                Ok((
                    build_game_server_key(region_system_name, &gs.game_server_id),
                    serde_json::to_string(gs)?,
                ))
            })
            .collect::<Result<Vec<_>, Error>>()?;

        self.client.mset(items).await?;
        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn update_game_servers<'a, I: ExactSizeIterator<Item = &'a GameServer>>(
        &self,
        region_system_name: &str,
        game_servers: I,
    ) -> Result<(), Error> {
        if game_servers.len() == 0 {
            return Ok(());
        }

        let items = game_servers
            .map(|gs| {
                Ok((
                    build_game_server_key(region_system_name, &gs.game_server_id),
                    serde_json::to_string(gs)?,
                ))
            })
            .collect::<Result<Vec<_>, Error>>()?;

        self.client.mset(items).await?;

        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn delete_game_servers<'a, I: ExactSizeIterator<Item = &'a GameServerId> + Clone>(
        &self,
        region_system_name: &str,
        game_server_ids: I,
    ) -> Result<(), Error> {
        if game_server_ids.len() == 0 {
            return Ok(());
        }

        let results = tokio::join!(
            self.delete_game_server_ids(region_system_name, game_server_ids.clone()),
            self.delete_game_server_values(region_system_name, game_server_ids)
        );

        results.0?;
        results.1?;

        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn delete_game_server_ids<'a, I: Iterator<Item = &'a GameServerId>>(
        &self,
        region_system_name: &str,
        game_server_ids: I,
    ) -> Result<(), Error> {
        let key = build_game_server_set_key(region_system_name);
        let members = game_server_ids.copied().collect::<Vec<_>>();
        self.client.srem(key, members).await?;
        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn delete_game_server_values<'a, I: ExactSizeIterator<Item = &'a GameServerId>>(
        &self,
        region_system_name: &str,
        game_server_ids: I,
    ) -> Result<(), Error> {
        let num_servers = game_server_ids.len();

        if num_servers == 0 {
            return Ok(());
        }

        let keys = game_server_ids
            .map(|id| build_game_server_key(region_system_name, id))
            .collect::<Vec<_>>();

        let deleted = self.client.del(keys).await?;
        if deleted != num_servers {
            log::error!("Cannot delete game servers");
        }

        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn reset(&self, region_system_name: &str) -> Result<(), Error> {
        let game_server_ids: Vec<GameServerId> = self
            .client
            .smembers(build_game_server_set_key(region_system_name))
            .await?;

        let num_servers = game_server_ids.len();

        if num_servers == 0 {
            return Ok(());
        }

        let keys = game_server_ids
            .iter()
            .map(|id| build_game_server_key(region_system_name, id))
            .collect::<Vec<_>>();

        let deleted = self.client.del(keys).await?;
        if deleted != num_servers {
            log::error!("Cannot delete game servers");
        }
        
        Ok(())
    }
}

//-------------------------------------------------------------------------------------------------
fn build_game_server_key(region_system_name: &str, game_server_id: &GameServerId) -> String {
    format!("{region_system_name}:gs:{game_server_id}")
}

//-------------------------------------------------------------------------------------------------
fn build_game_server_set_key(region_system_name: &str) -> String {
    format!("{region_system_name}:gss")
}
