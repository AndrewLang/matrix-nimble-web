#![cfg(feature = "redis")]

use std::collections::HashMap;

use nimble_web::config::redis::RedisConfig;
use nimble_web::config::Configuration;
use nimble_web::di::ServiceContainer;
use nimble_web::redis::{
    redis_cache::RedisCache, redis_client::RedisClient, redis_lock::RedisLock, RedisModule,
};

#[test]
fn redis_module_registers_client_cache_and_lock() {
    let mut container = ServiceContainer::new();
    let config = Configuration::default();
    RedisModule::register(&mut container, &config);

    let services = container.build();
    assert!(services.resolve::<RedisClient>().is_some());
    assert!(services.resolve::<RedisCache>().is_some());
    assert!(services.resolve::<RedisLock>().is_some());
    assert!(services.resolve::<RedisConfig>().is_some());
}

#[test]
fn redis_module_exposes_configuration_from_provider() {
    let mut values = HashMap::new();
    values.insert("redis.url".to_string(), "redis://example".to_string());
    values.insert("redis.pool_size".to_string(), "4".to_string());
    let config = Configuration::from_values(values);
    let mut container = ServiceContainer::new();
    RedisModule::register(&mut container, &config);

    let services = container.build();
    let stored = services
        .resolve::<RedisConfig>()
        .expect("redis config registered");
    assert_eq!(stored.url, "redis://example");
    assert_eq!(stored.pool_size, 4);
}
