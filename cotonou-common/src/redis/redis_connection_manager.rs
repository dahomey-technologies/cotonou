use crate::redis::{Error, RedisConfig};
use std::{collections::HashMap, result};

pub struct RedisConnectionManager {
    clients: HashMap<String, rustis::client::Client>,
}

type Result<T> = result::Result<T, Error>;

impl RedisConnectionManager {
    pub async fn initialize(redis_config: RedisConfig) -> Result<Self> {
        let mut clients_by_connection_string = HashMap::<String, rustis::client::Client>::new();
        let mut clients_by_name = HashMap::<String, rustis::client::Client>::new();

        for (name, connection_string) in &redis_config.connection_strings {
            let client: rustis::client::Client;

            if let Some(existing_client) = clients_by_connection_string.get(connection_string) {
                client = existing_client.clone();
            } else {
                client = rustis::client::Client::connect(connection_string.clone()).await?;
                clients_by_connection_string.insert(connection_string.clone(), client.clone());
            }

            clients_by_name.insert(name.clone(), client);
        }

        Ok(Self {
            clients: clients_by_name,
        })
    }

    pub fn get_client(&self, name: &str) -> Option<rustis::client::Client> {
        self.clients.get(name).cloned()
    }
}
