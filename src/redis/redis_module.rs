#![cfg(feature = "redis")]

use crate::config::{redis::RedisConfig, Configuration};
use crate::di::ServiceContainer;
use crate::redis::redis_cache::RedisCache;
use crate::redis::redis_client::RedisClient;
use crate::redis::redis_lock::RedisLock;

pub struct RedisModule;

impl RedisModule {
    pub fn register(container: &mut ServiceContainer, config: &Configuration) {
        let redis_config = config.redis_config();
        let client = RedisClient::new(&redis_config.url, redis_config.pool_size);
        let redis_config_clone = redis_config.clone();
        let client_clone = client.clone();

        container.register_singleton::<RedisConfig, _>(move |_| redis_config_clone.clone());
        container.register_singleton::<RedisClient, _>(move |_| client_clone.clone());

        container.register_singleton::<RedisCache, _>(move |provider| {
            let client = provider
                .resolve::<RedisClient>()
                .expect("redis client missing");
            RedisCache::new(client.as_ref().clone())
        });

        container.register_singleton::<RedisLock, _>(move |provider| {
            let client = provider
                .resolve::<RedisClient>()
                .expect("redis client missing");
            RedisLock::new(client.as_ref().clone())
        });
    }
}
