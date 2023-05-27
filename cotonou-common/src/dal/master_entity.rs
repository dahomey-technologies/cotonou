use crate::mongo_db_collection::MongoDbCollection;
use mongodb::bson::DateTime;

pub const KEY: &str = "_id";
pub const DATA_VERSION_PROPERTY: &str = "dv";
pub const ENTITY_VERSION_PROPERTY: &str = "ev";
pub const CREATION_DATE_PROPERTY: &str = "cd";
pub const LAST_MODIFICATION_DATE_PROPERTY: &str = "lmd";

pub trait MasterEntity<TI>: MongoDbCollection {
    fn get_id(&self) -> TI;
    fn set_id(&mut self, id: TI);

    fn get_data_version(&self) -> Option<u32>;
    fn set_data_version(&mut self, data_version: Option<u32>);

    fn get_creation_date(&self) -> DateTime;
    fn set_creation_date(&mut self, creation_date: DateTime);

    fn get_last_modification_date(&self) -> DateTime;
    fn set_last_modification_date(&mut self, last_modification_date: DateTime);
}
