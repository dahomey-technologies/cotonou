use crate::{Error, MatchmakingDAL};
use cotonou_common::{
    matchmaking::{GameServer, GameServerId, MatchmakingSession, MatchmakingTicket, SessionId},
    profile::ProfileId,
};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    hash::Hash,
    iter::FusedIterator,
};

//-------------------------------------------------------------------------------------------------
pub struct ItemCache<I, T, DAL>
where
    DAL: MatchmakingDAL<I, T> + Sync,
    T: Item<I> + Sync,
{
    region_system_name: String,
    matchmaking_dal: DAL,
    items: HashMap<I, T>,
    items_to_create: HashSet<I>,
    items_to_update: HashSet<I>,
    items_to_delete: HashSet<I>,
}

impl<I, T, DAL> ItemCache<I, T, DAL>
where
    DAL: MatchmakingDAL<I, T> + Sync,
    T: Item<I> + Sync,
    I: Eq + Hash + Clone + Display + Send + Sync,
{
    //-------------------------------------------------------------------------------------------------
    pub fn new(region_system_name: &str, matchmaking_dal: DAL) -> Self {
        Self {
            region_system_name: region_system_name.to_owned(),
            matchmaking_dal,
            items: HashMap::new(),
            items_to_create: HashSet::new(),
            items_to_update: HashSet::new(),
            items_to_delete: HashSet::new(),
        }
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn load(&mut self) -> Result<(), Error> {
        let items = self.matchmaking_dal.get(&self.region_system_name).await?;
        for item in items {
            self.items.insert(item.get_id().clone(), item);
        }

        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn save(&mut self) -> Result<(), Error> {
        let results = tokio::join!(
            self.matchmaking_dal.create(
                &self.region_system_name,
                self.items_to_create
                    .iter()
                    .map(|id| self.items.get(id).unwrap())
            ),
            self.matchmaking_dal.update(
                &self.region_system_name,
                self.items_to_update
                    .iter()
                    .map(|id| self.items.get(id).unwrap())
            ),
            self.matchmaking_dal
                .delete(&self.region_system_name, self.items_to_delete.iter())
        );

        results.0?;
        results.1?;
        results.2?;

        self.items_to_create.clear();
        self.items_to_update.clear();
        self.items_to_delete.clear();

        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    pub async fn reset(&mut self) -> Result<(), Error> {
        self.items.clear();
        self.items_to_create.clear();
        self.items_to_update.clear();
        self.items_to_delete.clear();

        self.matchmaking_dal.reset(&self.region_system_name).await?;

        Ok(())
    }

    //-------------------------------------------------------------------------------------------------
    pub fn get(&self, id: &I) -> Option<&T> {
        self.items.get(id)
    }

    //-------------------------------------------------------------------------------------------------
    pub fn get_mut(&mut self, id: &I) -> Option<&mut T> {
        self.items.get_mut(id)
    }

    //-------------------------------------------------------------------------------------------------
    pub fn create(&mut self, item: T) -> bool {
        let id = item.get_id().clone();
        self.items.insert(id.clone(), item);

        self.items_to_delete.remove(&id); // just in case it was deleted before being created again
        self.items_to_create.insert(id)
    }

    //-------------------------------------------------------------------------------------------------
    pub fn update(&mut self, id: I) {
        if !self.items_to_create.contains(&id) {
            self.items_to_update.insert(id);
        }
    }

    //-------------------------------------------------------------------------------------------------
    pub fn delete(&mut self, id: &I) -> Option<T> {
        let Some(item) = self.items.remove(id) else {
            log::warn!("Cannot find item {id} to delete in cache");
            return None;
        };

        if !self.items_to_create.remove(id) {
            self.items_to_update.remove(id); // just in case it was updated before being deleted
            self.items_to_delete.insert(id.clone());
        }

        Some(item)
    }

    pub fn iter(&self) -> Iter<'_, I, T> {
        Iter {
            inner: self.items.values(),
        }
    }
}

//-------------------------------------------------------------------------------------------------
pub struct Iter<'a, I, T> {
    inner: std::collections::hash_map::Values<'a, I, T>,
}

impl<'a, I, T> Iterator for Iter<'a, I, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.inner.len()
    }
}

impl<I, T> FusedIterator for Iter<'_, I, T> {}

impl<I, T> ExactSizeIterator for Iter<'_, I, T> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

//-------------------------------------------------------------------------------------------------
pub trait Item<I> {
    fn get_id(&self) -> &I;
}

impl Item<GameServerId> for GameServer {
    fn get_id(&self) -> &GameServerId {
        &self.game_server_id
    }
}

impl Item<ProfileId> for MatchmakingTicket {
    fn get_id(&self) -> &ProfileId {
        &self.owner_profile_id
    }
}

impl Item<SessionId> for MatchmakingSession {
    fn get_id(&self) -> &SessionId {
        &self.session_id
    }
}
