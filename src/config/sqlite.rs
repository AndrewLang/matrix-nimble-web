use crate::config::Configuration;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqliteConfig {
    pub url: String,
    pub pool_size: u32,
    pub timeout: u64,
}

impl Default for SqliteConfig {
    fn default() -> Self {
        Self {
            url: "sqlite://photon.db?mode=rwc".to_string(),
            pool_size: 10,
            timeout: 5,
        }
    }
}

impl SqliteConfig {
    pub fn from_configuration(config: &Configuration) -> Self {
        let mut sqlite = Self::default();
        if let Some(url) = config.get("SQLite.Url") {
            sqlite.url = url.to_string();
        }
        if let Some(pool) = config.get("SQLite.PoolSize").and_then(|v| v.parse().ok()) {
            sqlite.pool_size = pool;
        }
        if let Some(timeout) = config.get("SQLite.Timeout").and_then(|v| v.parse().ok()) {
            sqlite.timeout = timeout;
        }
        sqlite
    }
}
