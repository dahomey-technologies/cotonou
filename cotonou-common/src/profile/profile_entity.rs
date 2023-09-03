use crate::{
    database::{MasterEntity, MongoDbCollection},
    types::ProfileId,
};
use bson::DateTime;
use serde::{Deserialize, Serialize};

pub const TABLE_NAME: &str = "Profile";
pub const DISPLAY_NAME_PROPERTY: &str = "dn";
pub const PLATFORM_ID_PROPERTY: &str = "pi";
pub const ELOS_PROPERTY: &str = "elos";
pub const NUM_MATCHES_PLAYED_PROPERTY: &str = "nmp";

#[derive(Serialize, Deserialize)]
pub struct ProfileEntity {
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

impl MongoDbCollection for ProfileEntity {
    fn get_collection_name() -> &'static str {
        TABLE_NAME
    }
}

impl MasterEntity<ProfileId> for ProfileEntity {
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
