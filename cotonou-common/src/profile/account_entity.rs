use crate::{models::ProfileId, mongo_db_collection::MongoDbCollection};
use mongodb::bson::DateTime;
use serde::{Deserialize, Serialize};

const TABLE_NAME: &str = "Account";

#[derive(Serialize, Deserialize)]
pub struct AccountEntity {
    #[serde(rename = "_id")]
    pub platform_id: String,

    #[serde(rename = "pid")]
    pub profile_id: ProfileId,

    #[serde(rename = "cd")]
    pub creation_date: DateTime,
}

impl MongoDbCollection for AccountEntity {
    fn get_collection_name() -> &'static str {
        TABLE_NAME
    }
}
