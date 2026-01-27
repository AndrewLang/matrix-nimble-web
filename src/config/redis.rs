use crate::config::Configuration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: usize,
    pub default_ttl_seconds: u64,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            pool_size: 16,
            default_ttl_seconds: 60,
        }
    }
}

impl RedisConfig {
    pub fn from_configuration(config: &Configuration) -> Self {
        let mut cfg = Self::default();
        if let Some(url) = config.get("redis.url") {
            cfg.url = url.to_string();
        }
        if let Some(pool) = config.get("redis.pool_size").and_then(|v| v.parse().ok()) {
            cfg.pool_size = pool;
        }
        if let Some(ttl) = config
            .get("redis.default_ttl_seconds")
            .and_then(|v| v.parse().ok())
        {
            cfg.default_ttl_seconds = ttl;
        }
        cfg
    }
}
