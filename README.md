# nimble-web

Framework-oriented Rust library scaffold for web applications.

## DI usage

```rust
use nimble_web::di::ServiceContainer;

#[derive(Debug)]
struct Counter {
    id: u32,
}

let mut container = ServiceContainer::new();
container.register_singleton(|_| Counter { id: 1 });

let provider = container.build();
let scope = provider.create_scope();
let counter = scope.resolve::<Counter>().expect("registered");

println!("{}", counter.id);
```

Lifetimes: singleton (one per container), scoped (one per scope), transient (new per resolve).

## Redis support *(optional)*

Enable Redis extensions via `cargo build --features redis` / `cargo test --features redis`. When the feature is active:

- `RedisConfig` is constructed from the existing config pipeline (`redis.url`, `redis.pool_size`, `redis.default_ttl_seconds`).
- `RedisModule` registers `RedisClient`, `RedisCache`, and `RedisLock` into the DI container, so your services/controllers can resolve them just like any other dependency.
- `RedisCache`/`RedisLock` implement the `Cache` and `DistributedLock` traits, offering async JSON storage (with TTL) and safe distributed locking.
- Integration tests under `tests/redis_integration.rs` look for `REDIS_URL`; if the env var is missing they return early so the suite still passes without a live Redis.

### Sample usage

```rust
use nimble_web::redis::redis_cache::{Cache, RedisCache};
use nimble_web::redis::redis_lock::{DistributedLock, RedisLock};

async fn snapshot(cache: &RedisCache, lock: &RedisLock) {
    let token = lock.try_lock("photos:write", 60).await.unwrap();
    cache
        .set_json("photos:latest", &vec!["sunset".to_string()], None)
        .await
        .unwrap();
    let latest: Option<Vec<String>> = cache.get_json("photos:latest").await.unwrap();
    println!("latest snapshot = {:?}", latest);
    lock.unlock("photos:write", &token.unwrap()).await.unwrap();
}
```
