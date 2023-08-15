use crate::online_config::MongoDbConnectionStringProvider;

pub struct OnlineConfigManager {}

impl OnlineConfigManager {
    pub fn new() -> Self {
        Self {}
    }
}

impl MongoDbConnectionStringProvider for OnlineConfigManager {
    fn get_connection_string(&self) -> String {
        todo!()
    }
}
