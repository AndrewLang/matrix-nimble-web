#![cfg_attr(feature = "redis", allow(dead_code))]

use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[cfg(feature = "redis")]
use crate::redis::redis_client::RedisClient;

#[cfg(feature = "redis")]
use deadpool::managed::PoolError;
#[cfg(feature = "redis")]
use deadpool_redis::redis::{cmd, RedisError};

static TOKEN_SEQ: AtomicU64 = AtomicU64::new(1);

fn gen_token() -> String {
    TOKEN_SEQ.fetch_add(1, Ordering::Relaxed).to_string()
}

#[derive(Debug)]
pub enum LockError {
    #[cfg(feature = "redis")]
    Redis(RedisError),
    #[cfg(feature = "redis")]
    Pool(PoolError<RedisError>),
    Conflict(String),
}

impl fmt::Display for LockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(feature = "redis")]
            LockError::Redis(err) => write!(f, "redis error: {err}"),
            #[cfg(feature = "redis")]
            LockError::Pool(err) => write!(f, "pool error: {err}"),
            LockError::Conflict(msg) => write!(f, "lock conflict: {msg}"),
        }
    }
}

impl std::error::Error for LockError {}

#[cfg(feature = "redis")]
impl From<RedisError> for LockError {
    fn from(err: RedisError) -> Self {
        LockError::Redis(err)
    }
}

#[cfg(feature = "redis")]
impl From<PoolError<RedisError>> for LockError {
    fn from(err: PoolError<RedisError>) -> Self {
        LockError::Pool(err)
    }
}

#[async_trait]
pub trait DistributedLock: Send + Sync {
    async fn try_lock(&self, key: &str, ttl_secs: u64) -> Result<Option<String>, LockError>;
    async fn unlock(&self, key: &str, token: &str) -> Result<(), LockError>;
}

#[cfg(feature = "redis")]
pub struct RedisLock {
    client: RedisClient,
}

#[cfg(feature = "redis")]
impl RedisLock {
    pub fn new(client: RedisClient) -> Self {
        Self { client }
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl DistributedLock for RedisLock {
    async fn try_lock(&self, key: &str, ttl_secs: u64) -> Result<Option<String>, LockError> {
        let mut conn = self.client.get().await.map_err(LockError::from)?;
        let token = gen_token();
        let response: Option<String> = cmd("SET")
            .arg(key)
            .arg(&token)
            .arg("NX")
            .arg("EX")
            .arg(ttl_secs)
            .query_async(&mut conn)
            .await
            .map_err(LockError::from)?;
        Ok(response.map(|_| token))
    }

    async fn unlock(&self, key: &str, token: &str) -> Result<(), LockError> {
        let script = r#"
            if redis.call("GET", KEYS[1]) == ARGV[1] then
                return redis.call("DEL", KEYS[1])
            else
                return 0
            end
        "#;
        let mut conn = self.client.get().await.map_err(LockError::from)?;
        let _: i32 = cmd("EVAL")
            .arg(script)
            .arg(1)
            .arg(key)
            .arg(token)
            .query_async(&mut conn)
            .await
            .map_err(LockError::from)?;
        Ok(())
    }
}

pub struct InMemoryLock {
    entries: Arc<Mutex<HashMap<String, (String, Instant)>>>,
}

impl InMemoryLock {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl DistributedLock for InMemoryLock {
    async fn try_lock(&self, key: &str, ttl_secs: u64) -> Result<Option<String>, LockError> {
        let now = Instant::now();
        let deadline = now + Duration::from_secs(ttl_secs);
        let mut entries = self.entries.lock().await;

        if let Some((_, expiry)) = entries.get(key) {
            if Instant::now() < *expiry {
                return Ok(None);
            }
            entries.remove(key);
        }

        let token = gen_token();
        entries.insert(key.to_string(), (token.clone(), deadline));
        Ok(Some(token))
    }

    async fn unlock(&self, key: &str, token: &str) -> Result<(), LockError> {
        let mut entries = self.entries.lock().await;
        if let Some((stored, _)) = entries.get(key) {
            if stored == token {
                entries.remove(key);
            } else {
                return Err(LockError::Conflict("invalid token".into()));
            }
        }
        Ok(())
    }
}
