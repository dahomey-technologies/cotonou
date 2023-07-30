use crate::{
    database::MasterEntity, mongo_db::MongoDbCollection, profile::profile_entity,
    profile::Platform, types::ProfileId,
};
use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CoreProfileEntity {
    #[serde(rename = "_id")]
    pub id: ProfileId,
    #[serde(rename = "dv")]
    pub data_version: Option<u32>,
    #[serde(rename = "ev")]
    pub entity_version: u32,
    #[serde(rename = "cd")]
    pub creation_date: DateTime,
    #[serde(rename = "lmd")]
    pub last_modification_date: DateTime,
    #[serde(rename = "dn")]
    pub display_name: String,
    #[serde(rename = "pi")]
    pub platform_id: String,
}

impl CoreProfileEntity {
    pub fn new(id: ProfileId, platform_id: &str, display_name: &str) -> Self {
        Self {
            id,
            data_version: None,
            entity_version: 0,
            creation_date: DateTime::MIN,
            last_modification_date: DateTime::MIN,
            display_name: display_name.to_string(),
            platform_id: platform_id.to_string(),
        }
    }

    pub fn get_platform(&self) -> Platform {
        match self.platform_id.as_str() {
            "nul" => Platform::PC,
            _ => Platform::None,
        }
    }
}

impl MongoDbCollection for CoreProfileEntity {
    fn get_collection_name() -> &'static str {
        profile_entity::TABLE_NAME
    }
}

impl MasterEntity<ProfileId> for CoreProfileEntity {
    fn get_id(&self) -> ProfileId {
        self.id
    }

    fn set_id(&mut self, id: ProfileId) {
        self.id = id;
    }

    fn get_data_version(&self) -> Option<u32> {
        self.data_version
    }

    fn set_data_version(&mut self, data_version: Option<u32>) {
        self.data_version = data_version;
    }

    fn get_creation_date(&self) -> DateTime {
        self.creation_date
    }

    fn set_creation_date(&mut self, creation_date: DateTime) {
        self.creation_date = creation_date;
    }

    fn get_last_modification_date(&self) -> DateTime {
        self.last_modification_date
    }

    fn set_last_modification_date(&mut self, last_modification_date: DateTime) {
        self.last_modification_date = last_modification_date;
    }
}
