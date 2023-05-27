use std::collections::HashMap;

#[derive(Clone)]
pub struct RedisConfig {
    pub connection_strings: HashMap<String, String>,
}
