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

## Postgres support *(optional)*

Enable Postgres extensions via `cargo build --features postgres` / `cargo test --features postgres`.

### Sample usage

```rust
use nimble_web::AppBuilder;
use nimble_web::data::postgres::PostgresEntity;
use nimble_web::data::schema::{ColumnDef, ColumnType};
use nimble_web::data::query::Value;
use nimble_web::entity::entity::Entity;
use serde::{Deserialize, Serialize};

// 1. Define your entity
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
}

// 2. Implement Entity trait (core)
impl Entity for User {
    type Id = i64;
    fn id(&self) -> &i64 { &self.id }
    fn name() -> &'static str { "user" }
    fn plural_name() -> String { "users".to_string() }
}

// 3. Implement PostgresEntity
impl PostgresEntity for User {
    fn id_column() -> &'static str { "id" }
    fn id_value(id: &i64) -> Value { Value::Int(*id) }
    
    // Columns to insert
    fn insert_columns() -> &'static [&'static str] { &["id", "username", "email"] }
    fn insert_values(&self) -> Vec<Value> {
        vec![Value::Int(self.id), Value::String(self.username.clone()), Value::String(self.email.clone())]
    }

    // Columns to update
    fn update_columns() -> &'static [&'static str] { &["username", "email"] }
    fn update_values(&self) -> Vec<Value> {
        vec![Value::String(self.username.clone()), Value::String(self.email.clone())]
    }

    // Schema definition for migration
    fn table_columns() -> Vec<ColumnDef> {
        vec![
            ColumnDef::new("id", ColumnType::Integer).primary_key(),
            ColumnDef::new("username", ColumnType::Text).not_null().unique(),
            ColumnDef::new("email", ColumnType::Text).not_null(),
        ]
    }
}

// 4. Run migration
// 4. Integrate with AppBuilder
#[tokio::main]
async fn main() {
    // Application will load config from environment variables (NIMBLE_POSTGRES_URL)
    let mut builder = AppBuilder::new();
    
    // Load environment variables (e.g. NIMBLE_POSTGRES_URL=postgres://...)
    builder.use_env();
    
    // Enable Postgres (uses configuration)
    builder.use_postgres();
    
    // Register entity routes (CRUD)
    builder.use_entity::<User>();

    let app = builder.build();

    // Run migration at startup (optional)
    app.migrate_entity::<User>().await.expect("migration failed");

    app.start().await.expect("app failed");
}
```

Configuration keys:
- `postgres.url`: Connection string (default: `postgres://postgres:postgres@localhost:5432/postgres`)
- `postgres.pool_size`: Max connections (default: 10)
- `postgres.schema`: Optional schema prefix
