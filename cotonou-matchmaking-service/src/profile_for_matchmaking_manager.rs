use crate::{Error, ProfileForMatchmakingEntity};
use cotonou_common::{
    database::{master_entity, GenericDAL},
    profile::profile_entity,
    types::ProfileId,
};
use std::result;

#[derive(Clone)]
pub struct ProfileForMatchmakingManager {
    generic_dal: GenericDAL,
}

type Result<T> = result::Result<T, Error>;

impl ProfileForMatchmakingManager {
    pub fn new(generic_dal: GenericDAL) -> ProfileForMatchmakingManager {
        ProfileForMatchmakingManager { generic_dal }
    }

    pub async fn get_profiles_for_matchmaking(
        &self,
        profile_ids: &[ProfileId],
    ) -> Result<Vec<ProfileForMatchmakingEntity>> {
        let attributes_to_get = [
            master_entity::KEY,
            profile_entity::DISPLAY_NAME_PROPERTY,
            profile_entity::ELOS_PROPERTY,
            profile_entity::NUM_MATCHES_PLAYED_PROPERTY,
        ];

        Ok(self
            .generic_dal
            .get_partial_entities(profile_ids, &attributes_to_get)
            .await?)
    }

    // pub async fn get_profile_for_matchmaking(
    //     &self,
    //     profile_id: ProfileId,
    // ) -> Result<Option<ProfileForMatchmakingEntity>> {
    //     let attributes_to_get = [
    //         master_entity::KEY,
    //         profile_entity::ELOS_PROPERTY,
    //         profile_entity::NUM_MATCHES_PLAYED_PROPERTY,
    //     ];

    //     Ok(self
    //         .generic_dal
    //         .get_partial_entity(profile_id, &attributes_to_get)
    //         .await?)
    // }
}
