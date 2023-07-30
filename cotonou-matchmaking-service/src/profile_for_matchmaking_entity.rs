use cotonou_common::{
    profile::{profile_entity, ProfileId},
    mongo_db::MongoDbCollection,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct ProfileForMatchmakingEntity {
    #[serde(rename = "_id")]
    pub id: ProfileId,
    #[serde(rename = "dn")]
    pub display_name: String,
    /// Matchmaking ratings indexed by Game mode name
    #[serde(rename = "mmrs", default)]
    pub mmrs: HashMap<String, u32>,
    #[serde(rename = "nmp", default)]
    pub num_matches_played: u32,
}

impl MongoDbCollection for ProfileForMatchmakingEntity {
    fn get_collection_name() -> &'static str {
        profile_entity::TABLE_NAME
    }
}
