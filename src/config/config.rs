use std::collections::HashMap;

#[cfg(feature = "postgres")]
use crate::config::postgres::PostgresConfig;
#[cfg(feature = "redis")]
use crate::config::redis::RedisConfig;

#[derive(Clone, Debug, Default)]
pub struct Configuration {
    values: HashMap<String, String>,
}

impl Configuration {
    pub(crate) fn new(values: HashMap<String, String>) -> Self {
        Self { values }
    }

    pub fn from_values(values: HashMap<String, String>) -> Self {
        Self { values }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|value| value.as_str())
    }

    pub fn get_u64(&self, key: &str) -> Option<u64> {
        self.get(key)?.parse().ok()
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.get(key)? {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    }

    #[cfg(feature = "redis")]
    pub fn redis_config(&self) -> RedisConfig {
        RedisConfig::from_configuration(self)
    }

    #[cfg(feature = "postgres")]
    pub fn postgres_config(&self) -> PostgresConfig {
        PostgresConfig::from_configuration(self)
    }
}
