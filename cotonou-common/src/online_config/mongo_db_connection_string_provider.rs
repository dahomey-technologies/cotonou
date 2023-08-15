pub trait MongoDbConnectionStringProvider {
    fn get_connection_string(&self) -> String;
}