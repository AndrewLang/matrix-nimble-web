#![cfg(feature = "redis")]

use nimble_web::redis::redis_client::RedisClient;

#[test]
fn new_uses_configuration() {
    let client = RedisClient::new("redis://127.0.0.1:6379", 8);
    assert_eq!(client.pool_size, 8);
    assert_eq!(client.url, "redis://127.0.0.1:6379");
}

#[test]
fn new_handles_zero_pool_size() {
    let client = RedisClient::new("redis://127.0.0.1:6379", 0);
    assert_eq!(client.pool_size, 0);
}
