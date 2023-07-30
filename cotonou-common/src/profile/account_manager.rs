use crate::{
    database::{GenericDAL, IdGeneratorDAL},
    profile::{profile_entity, AccountEntity, Error},
};
use mongodb::bson::DateTime;
use std::result;

type Result<T> = result::Result<T, Error>;

#[derive(Clone)]
pub struct AccountManager {
    pub id_generator_dal: IdGeneratorDAL,
    pub generic_dal: GenericDAL,
}

impl AccountManager {
    pub fn new(id_generator_dal: IdGeneratorDAL, generic_dal: GenericDAL) -> Self {
        Self {
            id_generator_dal,
            generic_dal,
        }
    }

    pub async fn get_account_entity(&self, plaform_id: &str) -> Result<Option<AccountEntity>> {
        let result = self.generic_dal.get_entity(plaform_id.to_string()).await?;
        Ok(result)
    }

    pub async fn create_account_entity(&self, plaform_id: &str) -> Result<AccountEntity> {
        let profile_id = self
            .id_generator_dal
            .next_id(profile_entity::TABLE_NAME, 1)
            .await?
            .try_into()?;

        let mut account_entity = AccountEntity {
            platform_id: plaform_id.to_string(),
            profile_id,
            creation_date: DateTime::now(),
        };

        self.generic_dal.save_entity(&mut account_entity).await?;

        Ok(account_entity)
    }
}
