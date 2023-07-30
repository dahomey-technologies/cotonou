use crate::{
    matchmaking::{Error, MatchmakingTicket},
    types::ProfileId,
    redis::RedisConnectionManager,
};
use rustis::{
    client::Client,
    commands::{GenericCommands, SortedSetCommands, StringCommands},
};
use std::iter::once;

#[derive(Clone)]
pub struct MatchmakingTicketDAL {
    client: Client,
}

impl MatchmakingTicketDAL {
    pub fn new(redis_connection_manager: &RedisConnectionManager) -> Self {
        Self {
            client: redis_connection_manager.get_client("MATCHMAKING").unwrap(),
        }
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn get_tickets(
        &self,
        region_system_name: &str,
    ) -> Result<Vec<MatchmakingTicket>, Error> {
        let ticket_queue_key = build_ticket_queue_key(region_system_name);

        let profile_ids: Vec<ProfileId> = self
            .client
            .zrange(ticket_queue_key, 0, -1, Default::default())
            .await?;

        if profile_ids.is_empty() {
            return Ok(Vec::new());
        }

        let ticket_keys = profile_ids
            .into_iter()
            .map(|id| build_ticket_key(region_system_name, id))
            .collect::<Vec<_>>();
        let values: Vec<String> = self.client.mget(ticket_keys).await?;

        let tickets = values
            .into_iter()
            .map(|v| serde_json::from_str(&v))
            .collect::<serde_json::Result<Vec<MatchmakingTicket>>>()?;

        Ok(tickets)
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn get_ticket(
        &self,
        region_system_name: &str,
        owner_profile_id: ProfileId,
    ) -> Result<Option<MatchmakingTicket>, Error> {
        let ticket_key = build_ticket_key(region_system_name, owner_profile_id);

        let ticket_json: Option<String> = self.client.get(ticket_key).await?;
        if let Some(ticket_json) = ticket_json {
            Ok(Some(serde_json::from_str(&ticket_json)?))
        } else {
            Ok(None)
        }
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn create_tickets<'a, I: ExactSizeIterator<Item = &'a MatchmakingTicket> + Clone>(
        &self,
        region_system_name: &str,
        tickets: I,
    ) -> Result<(), Error> {
        if tickets.len() == 0 {
            return Ok(());
        }

        let results = tokio::join!(
            self.create_ticket_ids(region_system_name, tickets.clone()),
            self.create_ticket_values(region_system_name, tickets)
        );

        results.0?;
        results.1?;

        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    async fn create_ticket_ids<'a, I: Iterator<Item = &'a MatchmakingTicket>>(
        &self,
        region_system_name: &str,
        tickets: I,
    ) -> Result<(), Error> {
        let items = tickets
            .map(|t| (t.creation_time as f64, t.owner_profile_id))
            .collect::<Vec<_>>();
        let key = build_ticket_queue_key(region_system_name);
        self.client.zadd(key, items, Default::default()).await?;
        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    async fn create_ticket_values<'a, I: Iterator<Item = &'a MatchmakingTicket>>(
        &self,
        region_system_name: &str,
        tickets: I,
    ) -> Result<(), Error> {
        let items = tickets
            .map(|t| {
                Ok((
                    build_ticket_key(region_system_name, t.owner_profile_id),
                    serde_json::to_string(t)?,
                ))
            })
            .collect::<Result<Vec<_>, Error>>()?;

        self.client.mset(items).await?;
        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn update_tickets<'a, I: ExactSizeIterator<Item = &'a MatchmakingTicket>>(
        &self,
        region_system_name: &str,
        tickets: I,
    ) -> Result<(), Error> {
        if tickets.len() == 0 {
            return Ok(());
        }

        let items = tickets
            .map(|t| {
                Ok((
                    build_ticket_key(region_system_name, t.owner_profile_id),
                    serde_json::to_string(t)?,
                ))
            })
            .collect::<Result<Vec<_>, Error>>()?;

        self.client.mset(items).await?;

        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn delete_tickets<'a, I: ExactSizeIterator<Item = &'a ProfileId> + Clone>(
        &self,
        region_system_name: &str,
        ticket_ids: I,
    ) -> Result<(), Error> {
        if ticket_ids.len() == 0 {
            return Ok(());
        }

        let results = tokio::join!(
            self.delete_ticket_ids(region_system_name, ticket_ids.clone()),
            self.delete_ticket_values(region_system_name, ticket_ids)
        );

        results.0?;
        results.1?;

        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn delete_ticket_ids<'a, I: Iterator<Item = &'a ProfileId>>(
        &self,
        region_system_name: &str,
        ticket_ids: I,
    ) -> Result<(), Error> {
        let key = build_ticket_queue_key(region_system_name);
        let members = ticket_ids.copied().collect::<Vec<_>>();
        self.client.zrem(key, members).await?;
        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn delete_ticket_values<'a, I: ExactSizeIterator<Item = &'a ProfileId>>(
        &self,
        region_system_name: &str,
        ticket_ids: I,
    ) -> Result<(), Error> {
        let num_tickets = ticket_ids.len();
        let keys = ticket_ids
            .map(|id| build_ticket_key(region_system_name, *id))
            .collect::<Vec<_>>();
        let deleted = self.client.del(keys).await?;
        if deleted != num_tickets {
            log::error!("Cannot delete tickets");
        }
        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn reset(&self, region_system_name: &str) -> Result<(), Error> {
        let ticket_queue_key = build_ticket_queue_key(region_system_name);
        let ticket_ids: Vec<ProfileId> = self
            .client
            .zrange(ticket_queue_key.clone(), 0, -1, Default::default())
            .await?;

        let keys_to_delete = ticket_ids
            .into_iter()
            .map(|id| build_ticket_key(region_system_name, id))
            .chain(once(ticket_queue_key))
            .collect::<Vec<_>>();
        self.client.del(keys_to_delete).await?;
        Ok(())
    }
}

//-------------------------------------------------------------------------------------------------
#[inline]
fn build_ticket_queue_key(region_system_name: &str) -> String {
    format!("{region_system_name}:mmtq")
}

//-------------------------------------------------------------------------------------------------
#[inline]
fn build_ticket_key(region_system_name: &str, profile_id: ProfileId) -> String {
    format!("{region_system_name}:mmt:{profile_id}")
}
