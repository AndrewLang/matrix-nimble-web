#![cfg(feature = "redis")]

use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

use nimble_web::redis::redis_cache::{Cache, InMemoryCache};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Payload {
    value: String,
}

fn payload() -> Payload {
    Payload {
        value: "hello".into(),
    }
}

#[tokio::test]
async fn in_memory_cache_roundtrip() {
    let cache = InMemoryCache::new();
    cache
        .set_json("key", &payload(), None)
        .await
        .expect("set must work");
    let stored: Payload = cache
        .get_json("key")
        .await
        .expect("get succeeds")
        .expect("value exists");
    assert_eq!(stored, payload());
}

#[tokio::test]
async fn in_memory_cache_delete() {
    let cache = InMemoryCache::new();
    cache
        .set_json("key", &payload(), None)
        .await
        .expect("set must work");
    cache.delete("key").await.expect("delete works");
    assert!(cache.get_json::<Payload>("key").await.unwrap().is_none());
}

#[tokio::test]
async fn in_memory_cache_ttl_expires() {
    let cache = InMemoryCache::new();
    cache
        .set_json("key", &payload(), Some(1))
        .await
        .expect("set must work");
    sleep(Duration::from_secs(2)).await;
    let result = cache.get_json::<Payload>("key").await.unwrap();
    assert!(result.is_none());
}
