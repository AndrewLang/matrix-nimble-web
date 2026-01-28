use crate::config::Configuration;

#[derive(Clone, Debug, PartialEq, Eq)]
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
        if let Some(url) = config.get("postgres.url") {
            pg_config.url = url.to_string();
        }
        if let Some(pool) = config
            .get("postgres.pool_size")
            .and_then(|v| v.parse().ok())
        {
            pg_config.pool_size = pool;
        }
        if let Some(schema) = config.get("postgres.schema") {
            pg_config.schema = Some(schema.to_string());
        }
        pg_config
    }
}
