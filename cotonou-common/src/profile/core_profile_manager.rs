use crate::{
    database::{master_entity, GenericDAL},
    profile::{profile_entity, CoreProfileEntity, Error},
    types::ProfileId,
};
use std::result;

type Result<T> = result::Result<T, Error>;

#[derive(Clone)]
pub struct CoreProfileManager {
    pub generic_dal: GenericDAL,
}

impl CoreProfileManager {
    pub fn new(generic_dal: GenericDAL) -> Self {
        Self { generic_dal }
    }

    pub async fn create_core_profile(&self, core_profile: &mut CoreProfileEntity) -> Result<bool> {
        core_profile.entity_version = 1;
        Ok(self.generic_dal.save_master_entity(core_profile).await?)
    }

    pub async fn get_core_profile(
        &self,
        profile_id: ProfileId,
    ) -> Result<Option<CoreProfileEntity>> {
        let attributes_to_get = [
            master_entity::KEY,
            master_entity::DATA_VERSION_PROPERTY,
            master_entity::ENTITY_VERSION_PROPERTY,
            master_entity::CREATION_DATE_PROPERTY,
            master_entity::LAST_MODIFICATION_DATE_PROPERTY,
            profile_entity::DISPLAY_NAME_PROPERTY,
            profile_entity::PLATFORM_ID_PROPERTY,
        ];

        let core_profile = self
            .generic_dal
            .get_partial_entity(profile_id, &attributes_to_get)
            .await?;
        Ok(core_profile)
    }

    pub async fn update_display_name(
        &self,
        profile_id: ProfileId,
        display_name: &str,
    ) -> Result<()> {
        self.generic_dal
            .update_property::<CoreProfileEntity, _, _>(
                profile_id,
                profile_entity::DISPLAY_NAME_PROPERTY,
                display_name,
            )
            .await?;
        Ok(())
    }
}
