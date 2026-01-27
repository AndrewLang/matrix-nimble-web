#![cfg(feature = "redis")]

use std::collections::HashMap;

use nimble_web::config::{ConfigBuilder, Configuration};

#[test]
fn redis_config_defaults_when_missing_values() {
    let config = ConfigBuilder::new().build();
    let redis = config.redis_config();

    assert_eq!(redis.url, "redis://127.0.0.1:6379");
    assert_eq!(redis.pool_size, 16);
    assert_eq!(redis.default_ttl_seconds, 60);
}

#[test]
fn redis_config_parses_custom_values() {
    let mut values = HashMap::new();
    values.insert("redis.url".to_string(), "redis://example".to_string());
    values.insert("redis.pool_size".to_string(), "4".to_string());
    values.insert("redis.default_ttl_seconds".to_string(), "120".to_string());
    let config = Configuration::from_values(values);
    let redis = config.redis_config();

    assert_eq!(redis.url, "redis://example");
    assert_eq!(redis.pool_size, 4);
    assert_eq!(redis.default_ttl_seconds, 120);
}
