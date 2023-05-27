use crate::{generic_dal::{GenericDAL, self}, mongo_db_collection::MongoDbCollection};
use serde::{Deserialize, Serialize};

const TABLE_NAME: &str = "IdGenerator";
const NEXT_ID_PROPERTY: &str = "nid";

#[derive(Serialize, Deserialize)]
pub struct IdGenerator {
    #[serde(rename = "_id")]
    pub table_name: String,

    #[serde(rename = "nid")]
    pub next_id: String,
}

impl MongoDbCollection for IdGenerator {
    fn get_collection_name() -> &'static str {
        TABLE_NAME
    }
}

#[derive(Clone)]
pub struct IdGeneratorDAL {
    pub generic_dal: GenericDAL,
}

pub enum Error {
    Database,
}

impl From<generic_dal::Error> for Error {
    fn from(_: generic_dal::Error) -> Self {
        Error::Database
    }
}

impl IdGeneratorDAL {
    pub fn new(generic_dal: GenericDAL) -> Self {
        Self { generic_dal }
    }

    pub async fn next_id(&self, table_name: &str, count: i64) -> Result<i64, Error> {
        let result = self
            .generic_dal
            .increment_property::<IdGenerator, String>(
                table_name.to_string(),
                NEXT_ID_PROPERTY,
                count,
            )
            .await?
            .ok_or(Error::Database)?;

        Ok(result - count + 1)
    }
}
