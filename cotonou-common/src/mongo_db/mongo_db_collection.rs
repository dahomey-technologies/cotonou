pub trait MongoDbCollection {
    fn get_collection_name() -> &'static str;
}