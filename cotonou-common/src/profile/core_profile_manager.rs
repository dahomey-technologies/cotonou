use crate::{
    core_profile_entity::CoreProfileEntity,
    dal::generic_dal::GenericDAL,
    generic_dal,
    master_entity::{
        CREATION_DATE_PROPERTY, DATA_VERSION_PROPERTY, ENTITY_VERSION_PROPERTY, KEY,
        LAST_MODIFICATION_DATE_PROPERTY,
    },
    models::ProfileId,
    profile_entity,
};
use std::result;

pub type Result<T> = result::Result<T, Error>;

pub enum Error {
    Database,
}

impl From<generic_dal::Error> for Error {
    fn from(_: generic_dal::Error) -> Self {
        Error::Database
    }
}

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
        self.generic_dal
            .save_master_entity(core_profile)
            .await
            .or(Err(Error::Database))
    }

    pub async fn get_core_profile(
        &self,
        profile_id: ProfileId,
    ) -> Result<Option<CoreProfileEntity>> {
        let attributes_to_get = [
            KEY,
            DATA_VERSION_PROPERTY,
            ENTITY_VERSION_PROPERTY,
            CREATION_DATE_PROPERTY,
            LAST_MODIFICATION_DATE_PROPERTY,
            profile_entity::DISPLAY_NAME_PROPERTY,
            profile_entity::PLATFORM_ID_PROPERTY,
        ];

        let core_profile = self
            .generic_dal
            .get_partial_entity(profile_id, &attributes_to_get)
            .await?;
        Ok(core_profile)
    }

    pub async fn update_display_name(&self, profile_id: ProfileId, display_name: &str) -> Result<()> {
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
