use std::iter::once;

use crate::{
    matchmaking::matchmaking_session::{MatchmakingSession, SessionId},
    redis::redis_connection_manager::RedisConnectionManager,
};
use rustis::{
    client::Client,
    commands::{GenericCommands, SortedSetCommands, StringCommands},
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
pub struct MatchmakingSessionDAL {
    client: Client,
}

impl MatchmakingSessionDAL {
    //-------------------------------------------------------------------------------------------------
    pub fn new(redis_connection_manager: &RedisConnectionManager) -> Self {
        Self {
            client: redis_connection_manager.get_client("MATCHMAKING").unwrap(),
        }
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn get_sessions(
        &self,
        region_system_name: &str,
    ) -> Result<Vec<MatchmakingSession>, Error> {
        let session_session_key = build_session_queue_key(region_system_name);

        let session_ids: Vec<SessionId> = self
            .client
            .zrange(session_session_key, 0, -1, Default::default())
            .await?;

        if session_ids.is_empty() {
            return Ok(Vec::new());
        }

        let session_keys = session_ids
            .into_iter()
            .map(|id| build_session_key(region_system_name, &id))
            .collect::<Vec<_>>();
        let values: Vec<String> = self.client.mget(session_keys).await?;

        let sessions = values
            .into_iter()
            .map(|v| serde_json::from_str(&v))
            .collect::<serde_json::Result<Vec<MatchmakingSession>>>()?;

        Ok(sessions)
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn get_session(
        &self,
        region_system_name: &str,
        session_id: &SessionId,
    ) -> Result<Option<MatchmakingSession>, Error> {
        let session_key = build_session_key(region_system_name, session_id);

        let session_json: Option<String> = self.client.get(session_key).await?;
        if let Some(session_json) = session_json {
            Ok(Some(serde_json::from_str(&session_json)?))
        } else {
            Ok(None)
        }
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn create_sessions<
        'a,
        I: ExactSizeIterator<Item = &'a MatchmakingSession> + Clone,
    >(
        &self,
        region_system_name: &str,
        sessions: I,
    ) -> Result<(), Error> {
        if sessions.len() == 0 {
            return Ok(());
        }

        let results = tokio::join!(
            self.create_session_ids(region_system_name, sessions.clone()),
            self.create_session_values(region_system_name, sessions)
        );

        results.0?;
        results.1?;

        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    async fn create_session_ids<'a, I: Iterator<Item = &'a MatchmakingSession>>(
        &self,
        region_system_name: &str,
        sessions: I,
    ) -> Result<(), Error> {
        let items = sessions
            .map(|t| (t.creation_time as f64, t.session_id))
            .collect::<Vec<_>>();
        let key = build_session_queue_key(region_system_name);
        self.client.zadd(key, items, Default::default()).await?;
        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    async fn create_session_values<'a, I: Iterator<Item = &'a MatchmakingSession>>(
        &self,
        region_system_name: &str,
        sessions: I,
    ) -> Result<(), Error> {
        let items = sessions
            .map(|t| {
                Ok((
                    build_session_key(region_system_name, &t.session_id),
                    serde_json::to_string(t)?,
                ))
            })
            .collect::<Result<Vec<_>, Error>>()?;

        self.client.mset(items).await?;
        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn update_sessions<'a, I: ExactSizeIterator<Item = &'a MatchmakingSession>>(
        &self,
        region_system_name: &str,
        sessions: I,
    ) -> Result<(), Error> {
        if sessions.len() == 0 {
            return Ok(());
        }

        let items = sessions
            .map(|s| {
                Ok((
                    build_session_key(region_system_name, &s.session_id),
                    serde_json::to_string(s)?,
                ))
            })
            .collect::<Result<Vec<_>, Error>>()?;

        self.client.mset(items).await?;

        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn delete_sessions<'a, I: ExactSizeIterator<Item = &'a SessionId> + Clone>(
        &self,
        region_system_name: &str,
        session_ids: I,
    ) -> Result<(), Error> {
        if session_ids.len() == 0 {
            return Ok(());
        }

        let results = tokio::join!(
            self.delete_session_ids(region_system_name, session_ids.clone()),
            self.delete_session_values(region_system_name, session_ids)
        );

        results.0?;
        results.1?;

        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn delete_session_ids<'a, I: Iterator<Item = &'a SessionId>>(
        &self,
        region_system_name: &str,
        session_ids: I,
    ) -> Result<(), Error> {
        let key = build_session_queue_key(region_system_name);
        let members = session_ids.copied().collect::<Vec<_>>();
        self.client.zrem(key, members).await?;
        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn delete_session_values<'a, I: ExactSizeIterator<Item = &'a SessionId>>(
        &self,
        region_system_name: &str,
        session_ids: I,
    ) -> Result<(), Error> {
        let num_sessions = session_ids.len();
        let keys = session_ids
            .map(|id| build_session_key(region_system_name, id))
            .collect::<Vec<_>>();
        let deleted = self.client.del(keys).await?;
        if deleted != num_sessions {
            log::error!("Cannot delete sessions");
        }
        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn reset(&self, region_system_name: &str) -> Result<(), Error> {
        let session_queue_key = build_session_queue_key(region_system_name);
        let session_ids: Vec<SessionId> = self
            .client
            .zrange(session_queue_key.clone(), 0, -1, Default::default())
            .await?;

        let keys_to_delete = session_ids
            .into_iter()
            .map(|id| build_session_key(region_system_name, &id))
            .chain(once(session_queue_key))
            .collect::<Vec<_>>();
        self.client.del(keys_to_delete).await?;
        Ok(())
    }
}

//-------------------------------------------------------------------------------------------------
#[inline]
fn build_session_queue_key(region_system_name: &str) -> String {
    format!("{region_system_name}:mmsq")
}

//-------------------------------------------------------------------------------------------------
#[inline]
fn build_session_key(region_system_name: &str, session_id: &SessionId) -> String {
    format!("{region_system_name}:mms:{session_id}")
}
