use crate::Error;
use cotonou_common::{
    matchmaking::{
        GameServer, GameServerDAL, GameServerId, MatchmakingSession, MatchmakingSessionDAL,
        MatchmakingTicket, MatchmakingTicketDAL, SessionId,
    },
    profile::ProfileId,
};
use futures_util::future::BoxFuture;

pub trait MatchmakingDAL<Id, Item> {
    //-------------------------------------------------------------------------------------------------
    fn get<'a>(&'a self, region_system_name: &'a str) -> BoxFuture<'a, Result<Vec<Item>, Error>>
    where
        Item: 'a;

    //-------------------------------------------------------------------------------------------------
    fn create<'a, I>(
        &'a self,
        region_system_name: &'a str,
        items: I,
    ) -> BoxFuture<'a, Result<(), Error>>
    where
        Item: 'a,
        I: ExactSizeIterator<Item = &'a Item> + Clone + Send + 'a;

    //-------------------------------------------------------------------------------------------------
    fn update<'a, I>(
        &'a self,
        region_system_name: &'a str,
        items: I,
    ) -> BoxFuture<'a, Result<(), Error>>
    where
        Item: 'a,
        I: ExactSizeIterator<Item = &'a Item> + Clone + Send + 'a;

    //-------------------------------------------------------------------------------------------------
    fn delete<'a, I>(
        &'a self,
        region_system_name: &'a str,
        ids: I,
    ) -> BoxFuture<'a, Result<(), Error>>
    where
        Id: 'a,
        I: ExactSizeIterator<Item = &'a Id> + Clone + Send + 'a;

    //-------------------------------------------------------------------------------------------------
    fn reset<'a>(&'a self, region_system_name: &'a str) -> BoxFuture<'a, Result<(), Error>>;
}

impl MatchmakingDAL<SessionId, MatchmakingSession> for MatchmakingSessionDAL {
    //-------------------------------------------------------------------------------------------------
    fn get<'a>(
        &'a self,
        region_system_name: &'a str,
    ) -> BoxFuture<'a, Result<Vec<MatchmakingSession>, Error>>
    where
        MatchmakingSession: 'a,
    {
        Box::pin(async move { Ok(self.get_sessions(region_system_name).await?) })
    }

    //-------------------------------------------------------------------------------------------------
    fn create<'a, I>(
        &'a self,
        region_system_name: &'a str,
        items: I,
    ) -> BoxFuture<'a, Result<(), Error>>
    where
        MatchmakingSession: 'a,
        I: ExactSizeIterator<Item = &'a MatchmakingSession> + Clone + Send + 'a,
    {
        Box::pin(async move {
            self.create_sessions(region_system_name, items).await?;
            Ok(())
        })
    }

    //-------------------------------------------------------------------------------------------------
    fn update<'a, I>(
        &'a self,
        region_system_name: &'a str,
        items: I,
    ) -> BoxFuture<'a, Result<(), Error>>
    where
        MatchmakingSession: 'a,
        I: ExactSizeIterator<Item = &'a MatchmakingSession> + Clone + Send + 'a,
    {
        Box::pin(async move {
            self.update_sessions(region_system_name, items).await?;
            Ok(())
        })
    }

    //-------------------------------------------------------------------------------------------------
    fn delete<'a, I>(
        &'a self,
        region_system_name: &'a str,
        ids: I,
    ) -> BoxFuture<'a, Result<(), Error>>
    where
        SessionId: 'a,
        I: ExactSizeIterator<Item = &'a SessionId> + Clone + Send + 'a,
    {
        Box::pin(async move {
            self.delete_sessions(region_system_name, ids).await?;
            Ok(())
        })
    }

    //-------------------------------------------------------------------------------------------------
    fn reset<'a>(&'a self, region_system_name: &'a str) -> BoxFuture<'a, Result<(), Error>> {
        Box::pin(async move {
            self.reset(region_system_name).await?;
            Ok(())
        })
    }
}

impl MatchmakingDAL<ProfileId, MatchmakingTicket> for MatchmakingTicketDAL {
    //-------------------------------------------------------------------------------------------------
    fn get<'a>(
        &'a self,
        region_system_name: &'a str,
    ) -> BoxFuture<'a, Result<Vec<MatchmakingTicket>, Error>>
    where
        MatchmakingTicket: 'a,
    {
        Box::pin(async move { Ok(self.get_tickets(region_system_name).await?) })
    }

    //-------------------------------------------------------------------------------------------------
    fn create<'a, I>(
        &'a self,
        region_system_name: &'a str,
        items: I,
    ) -> BoxFuture<'a, Result<(), Error>>
    where
        MatchmakingTicket: 'a,
        I: ExactSizeIterator<Item = &'a MatchmakingTicket> + Clone + Send + 'a,
    {
        Box::pin(async move {
            self.create_tickets(region_system_name, items).await?;
            Ok(())
        })
    }

    //-------------------------------------------------------------------------------------------------
    fn update<'a, I>(
        &'a self,
        region_system_name: &'a str,
        items: I,
    ) -> BoxFuture<'a, Result<(), Error>>
    where
        MatchmakingTicket: 'a,
        I: ExactSizeIterator<Item = &'a MatchmakingTicket> + Clone + Send + 'a,
    {
        Box::pin(async move {
            self.update_tickets(region_system_name, items).await?;
            Ok(())
        })
    }

    //-------------------------------------------------------------------------------------------------
    fn delete<'a, I>(
        &'a self,
        region_system_name: &'a str,
        ids: I,
    ) -> BoxFuture<'a, Result<(), Error>>
    where
        ProfileId: 'a,
        I: ExactSizeIterator<Item = &'a ProfileId> + Clone + Send + 'a,
    {
        Box::pin(async move {
            self.delete_tickets(region_system_name, ids).await?;
            Ok(())
        })
    }

    //-------------------------------------------------------------------------------------------------
    fn reset<'a>(&'a self, region_system_name: &'a str) -> BoxFuture<'a, Result<(), Error>> {
        Box::pin(async move {
            self.reset(region_system_name).await?;
            Ok(())
        })
    }
}

impl MatchmakingDAL<GameServerId, GameServer> for GameServerDAL {
    //-------------------------------------------------------------------------------------------------
    fn get<'a>(
        &'a self,
        region_system_name: &'a str,
    ) -> BoxFuture<'a, Result<Vec<GameServer>, Error>>
    where
        GameServer: 'a,
    {
        Box::pin(async move { Ok(self.get_game_servers(region_system_name).await?) })
    }

    //-------------------------------------------------------------------------------------------------
    fn create<'a, I>(
        &'a self,
        region_system_name: &'a str,
        items: I,
    ) -> BoxFuture<'a, Result<(), Error>>
    where
        GameServer: 'a,
        I: ExactSizeIterator<Item = &'a GameServer> + Clone + Send + 'a,
    {
        Box::pin(async move {
            self.create_game_servers(region_system_name, items).await?;
            Ok(())
        })
    }

    //-------------------------------------------------------------------------------------------------
    fn update<'a, I>(
        &'a self,
        region_system_name: &'a str,
        items: I,
    ) -> BoxFuture<'a, Result<(), Error>>
    where
        GameServer: 'a,
        I: ExactSizeIterator<Item = &'a GameServer> + Clone + Send + 'a,
    {
        Box::pin(async move {
            self.update_game_servers(region_system_name, items).await?;
            Ok(())
        })
    }

    //-------------------------------------------------------------------------------------------------
    fn delete<'a, I>(
        &'a self,
        region_system_name: &'a str,
        ids: I,
    ) -> BoxFuture<'a, Result<(), Error>>
    where
        GameServerId: 'a,
        I: ExactSizeIterator<Item = &'a GameServerId> + Clone + Send + 'a,
    {
        Box::pin(async move {
            self.delete_game_servers(region_system_name, ids).await?;
            Ok(())
        })
    }

    //-------------------------------------------------------------------------------------------------
    fn reset<'a>(&'a self, region_system_name: &'a str) -> BoxFuture<'a, Result<(), Error>> {
        Box::pin(async move {
            self.reset(region_system_name).await?;
            Ok(())
        })
    }
}
