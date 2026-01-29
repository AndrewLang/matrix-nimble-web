use crate::config::Configuration;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresConfig {
    pub url: String,
    pub pool_size: u32,
    pub schema: Option<String>,
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            url: "postgres://postgres:postgres@localhost:5432/postgres".to_string(),
            pool_size: 10,
            schema: None,
        }
    }
}

impl PostgresConfig {
    pub fn from_configuration(config: &Configuration) -> Self {
        let mut pg_config = Self::default();
        if let Some(url) = config.get("Postgres.Url") {
            pg_config.url = url.to_string();
        }
        if let Some(pool) = config.get("Postgres.PoolSize").and_then(|v| v.parse().ok()) {
            pg_config.pool_size = pool;
        }
        if let Some(schema) = config.get("Postgres.Schema") {
            pg_config.schema = Some(schema.to_string());
        }

        log::debug!("PostgresConfig loaded: {:?}", pg_config);
        pg_config
    }
}
