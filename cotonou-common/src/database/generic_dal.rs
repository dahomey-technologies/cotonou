use crate::{
    database::{master_entity, Error, MasterEntity, MongoDbCollection},
    mongo_db::MongoDbConfig,
};
use futures::TryStreamExt;
use mongodb::{
    bson::{self, Bson, DateTime},
    options::{
        FindOneAndUpdateOptions, FindOneOptions, FindOptions, ReplaceOptions, ReturnDocument,
        UpdateModifications,
    },
};
use serde::{de::DeserializeOwned, Serialize};
use std::{convert::From, fmt::Display, result};

type Result<T> = result::Result<T, Error>;

const MAX_TRIES: usize = 5;

#[derive(Clone)]
pub struct GenericDAL {
    mongodb_database: mongodb::Database,
}

impl GenericDAL {
    pub async fn initialize(mongo_config: &MongoDbConfig) -> Result<GenericDAL> {
        let mongo_options =
            mongodb::options::ClientOptions::parse(&mongo_config.connection_string).await?;
        let database_name = mongo_options
            .default_database
            .clone()
            .ok_or(Error::Database)?;
        let mongo_client = mongodb::Client::with_options(mongo_options)?;
        let mongo_database = mongo_client.database(&database_name);

        Ok(Self {
            mongodb_database: mongo_database,
        })
    }

    pub async fn get_partial_entity<T, TI>(
        &self,
        entity_id: TI,
        attributes_to_get: &[&str],
    ) -> Result<Option<T>>
    where
        T: MongoDbCollection + DeserializeOwned + Unpin + Send + Sync,
        TI: Into<Bson>,
    {
        let mongo_collection = self.get_collection::<T>();
        let filter = bson::doc! { "_id": entity_id.into() };
        let options = FindOneOptions::builder()
            .projection(Self::get_projection(attributes_to_get))
            .build();
        Ok(mongo_collection.find_one(filter, options).await?)
    }

    pub async fn get_partial_entities<T, TI>(
        &self,
        entity_ids: &[TI],
        attributes_to_get: &[&str],
    ) -> Result<Vec<T>>
    where
        T: MongoDbCollection + DeserializeOwned + Unpin + Send + Sync,
        TI: Into<Bson>,
        TI: Clone,
    {
        let mongo_collection = self.get_collection::<T>();

        let ids = entity_ids.iter().map(|i| i.into()).collect::<Vec<Bson>>();
        let mut filter = bson::doc! { "_id": {  } };
        filter
            .get_document_mut("_id")
            .or(Err(Error::Database))?
            .insert("$in", ids);

        let options = FindOptions::builder()
            .projection(Self::get_projection(attributes_to_get))
            .build();

        Ok(mongo_collection
            .find(filter, options)
            .await?
            .try_collect()
            .await?)
    }

    pub async fn get_entity<T, TI>(&self, entity_id: TI) -> Result<Option<T>>
    where
        T: MongoDbCollection + DeserializeOwned + Unpin + Send + Sync,
        TI: Into<Bson>,
    {
        let mongo_collection = self.get_collection::<T>();
        let filter = bson::doc! { "_id": entity_id.into() };
        Ok(mongo_collection.find_one(filter, None).await?)
    }

    pub async fn save_master_entity<T, TI>(&self, entity: &mut T) -> Result<bool>
    where
        T: MasterEntity<TI> + Serialize + Unpin + Send + Sync,
        TI: Into<Bson>,
    {
        if entity.get_creation_date() == DateTime::MIN {
            entity.set_creation_date(DateTime::now());
            entity.set_data_version(Some(1));
        }

        entity.set_last_modification_date(entity.get_creation_date());

        let mut query = bson::Document::new();
        query.insert("_id", entity.get_id());
        if let Some(data_version) = entity.get_data_version() {
            query.insert(master_entity::DATA_VERSION_PROPERTY, data_version);
        }

        entity.set_data_version(Some(entity.get_data_version().unwrap_or(0) + 1));

        let options = ReplaceOptions::builder().upsert(true).build();

        let mongo_collection = self.get_collection::<T>();
        let result = mongo_collection.replace_one(query, entity, options).await?;

        Ok(result.modified_count == 1 || result.upserted_id.is_some())
    }

    pub async fn save_master_entity_with_comparison<T, TI>(
        &self,
        original_entity: Option<&T>,
        modified_entity: Option<&mut T>,
    ) -> Result<bool>
    where
        T: MasterEntity<TI> + Serialize + Unpin + Send + Sync + PartialEq,
        TI: Display + Into<Bson>,
    {
        let mongo_collection = self.get_collection::<T>();

        match (original_entity, modified_entity) {
            (None, None) => Ok(true),
            // creation
            (None, Some(modified_entity)) => {
                modified_entity.set_creation_date(DateTime::now());
                modified_entity.set_last_modification_date(DateTime::now());
                modified_entity.set_data_version(Some(1));

                let _result = mongo_collection.insert_one(modified_entity, None).await?;

                Ok(true)
            }
            // deletion
            (Some(original_entity), None) => {
                self.delete_entity::<T, TI>(original_entity.get_id())
                    .await?;
                log::info!("Entity {} has been deleted", original_entity.get_id());

                Ok(true)
            }
            // update
            (Some(original_entity), Some(modified_entity)) => {
                if original_entity == modified_entity {
                    return Ok(true);
                }

                let original_data_version = original_entity.get_data_version().unwrap_or(0);
                modified_entity.set_data_version(Some(original_data_version + 1));
                modified_entity.set_last_modification_date(DateTime::now());

                let filter = bson::doc! { "_id": modified_entity.get_id().into(), master_entity::DATA_VERSION_PROPERTY: original_data_version };
                let result = mongo_collection.replace_one(filter, modified_entity, None).await?;

                Ok(result.modified_count == 1)
            }
        }
    }

    pub async fn save_entity<T>(&self, entity: &mut T) -> Result<()>
    where
        T: MongoDbCollection + Serialize + Unpin + Send + Sync,
    {
        let mongo_collection = self.get_collection::<T>();
        let _result = mongo_collection.insert_one(entity, None).await?;
        Ok(())
    }

    pub async fn update_property<T, TI, TP>(
        &self,
        entity_id: TI,
        property_name: &str,
        new_value: TP,
    ) -> Result<()>
    where
        T: MongoDbCollection + Serialize + Unpin + Send + Sync,
        Bson: From<TI>,
        Bson: From<TP>,
    {
        let mongo_collection = self.get_collection::<T>();

        let query = bson::doc! { "_id": entity_id };
        let update =
            UpdateModifications::Document(bson::doc! { "$set": {property_name: new_value} });

        mongo_collection.update_one(query, update, None).await?;

        Ok(())
    }

    pub async fn delete_entity<T, TI>(&self, entity_id: TI) -> Result<bool>
    where
        T: MongoDbCollection,
        TI: Into<Bson>,
    {
        let mongo_collection = self.get_collection::<T>();
        let result = mongo_collection
            .delete_one(bson::doc! { "_id": entity_id.into() }, None)
            .await?;
        Ok(result.deleted_count == 1)
    }

    pub async fn increment_property<T, TI>(
        &self,
        entity_id: TI,
        property_name: &str,
        value: i64,
    ) -> Result<Option<i64>>
    where
        T: MongoDbCollection + Serialize + Unpin + Send + Sync,
        Bson: From<TI>,
    {
        let bson_value = self
            .increment_property_impl::<T, TI, i64>(entity_id, property_name, value)
            .await?;
        match bson_value {
            Some(Bson::Int64(value)) => Ok(Some(value)),
            _ => Ok(None),
        }
    }

    async fn increment_property_impl<T, TI, TP>(
        &self,
        entity_id: TI,
        property_name: &str,
        value: TP,
    ) -> Result<Option<bson::Bson>>
    where
        T: MongoDbCollection + Serialize + Unpin + Send + Sync,
        TI: Into<Bson>,
        TP: Into<Bson>,
    {
        let mongo_collection = self.get_bson_document_collection::<T>();

        let filter = bson::doc! { "_id": entity_id.into() };
        let update =
            UpdateModifications::Document(bson::doc! { "$inc": {property_name: value.into()} });
        let options = FindOneAndUpdateOptions::builder()
            .upsert(true)
            .return_document(ReturnDocument::After)
            .build();

        let document = mongo_collection
            .find_one_and_update(filter, update, options)
            .await?;

        match document {
            Some(document) => match document.get(property_name) {
                Some(bson) => Ok(Some(bson.clone())),
                None => Ok(None),
            },
            None => Ok(None),
        }
    }

    fn get_collection<T>(&self) -> mongodb::Collection<T>
    where
        T: MongoDbCollection,
    {
        self.mongodb_database.collection(T::get_collection_name())
    }

    fn get_bson_document_collection<T>(&self) -> mongodb::Collection<bson::Document>
    where
        T: MongoDbCollection,
    {
        self.mongodb_database.collection(T::get_collection_name())
    }

    fn get_projection(attributes_to_get: &[&str]) -> bson::Document {
        let mut doc = bson::Document::new();
        for attribute in attributes_to_get {
            doc.insert(*attribute, 1);
        }

        doc
    }
}
