#![cfg(feature = "redis")]

use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use nimble_web::redis::{
    redis_cache::{Cache, CacheError, RedisCache},
    redis_client::RedisClient,
    redis_lock::{DistributedLock, LockError, RedisLock},
};

fn redis_url() -> Option<String> {
    env::var("REDIS_URL").ok()
}

fn key_prefix() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    format!("nimble:test:{}:", timestamp)
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Payload {
    value: String,
}

#[tokio::test]
async fn redis_cache_json_roundtrip() -> Result<(), CacheError> {
    let url = match redis_url() {
        Some(url) => url,
        None => return Ok(()),
    };
    let client = RedisClient::new(&url, 2);
    let cache = RedisCache::new(client.clone());
    let prefix = key_prefix();
    let key = format!("{prefix}cache");
    let payload = Payload {
        value: "integration".to_string(),
    };

    cache.set_json(&key, &payload, Some(5)).await?;
    let stored = cache.get_json::<Payload>(&key).await?;
    assert_eq!(stored.unwrap(), payload);
    cache.delete(&key).await?;
    assert!(cache.get_json::<Payload>(&key).await?.is_none());
    Ok(())
}

#[tokio::test]
async fn redis_lock_blocks_and_unlocks() -> Result<(), LockError> {
    let url = match redis_url() {
        Some(url) => url,
        None => return Ok(()),
    };
    let client = RedisClient::new(&url, 2);
    let lock = RedisLock::new(client.clone());
    let key = format!("{}lock", key_prefix());

    let token = lock
        .try_lock(&key, 5)
        .await?
        .expect("initial lock acquires");
    assert!(lock.try_lock(&key, 5).await?.is_none());
    lock.unlock(&key, &token).await?;
    let second = lock.try_lock(&key, 5).await?;
    assert!(second.is_some());
    lock.unlock(&key, &second.unwrap()).await?;
    Ok(())
}
