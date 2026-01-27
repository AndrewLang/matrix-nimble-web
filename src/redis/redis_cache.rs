#![cfg_attr(feature = "redis", allow(dead_code))]

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[cfg(feature = "redis")]
use crate::redis::redis_client::RedisClient;

#[cfg(feature = "redis")]
use deadpool::managed::PoolError;
#[cfg(feature = "redis")]
use deadpool_redis::redis::{AsyncCommands, RedisError};

/// Errors that can occur when interacting with the cache.
#[derive(Debug)]
pub enum CacheError {
    Serialization(serde_json::Error),
    #[cfg(feature = "redis")]
    Redis(RedisError),
    #[cfg(feature = "redis")]
    Pool(PoolError<RedisError>),
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheError::Serialization(err) => write!(f, "serialization error: {err}"),
            #[cfg(feature = "redis")]
            CacheError::Redis(err) => write!(f, "redis error: {err}"),
            #[cfg(feature = "redis")]
            CacheError::Pool(err) => write!(f, "pool error: {err}"),
        }
    }
}

impl std::error::Error for CacheError {}

impl From<serde_json::Error> for CacheError {
    fn from(err: serde_json::Error) -> Self {
        CacheError::Serialization(err)
    }
}

#[cfg(feature = "redis")]
impl From<RedisError> for CacheError {
    fn from(err: RedisError) -> Self {
        CacheError::Redis(err)
    }
}

#[cfg(feature = "redis")]
impl From<PoolError<RedisError>> for CacheError {
    fn from(err: PoolError<RedisError>) -> Self {
        CacheError::Pool(err)
    }
}

#[async_trait]
pub trait Cache: Send + Sync {
    async fn get_json<T: DeserializeOwned + Send>(
        &self,
        key: &str,
    ) -> Result<Option<T>, CacheError>;
    async fn set_json<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl_secs: Option<u64>,
    ) -> Result<(), CacheError>;
    async fn delete(&self, key: &str) -> Result<(), CacheError>;
}

/// Redis-backed cache implementation.
#[cfg(feature = "redis")]
pub struct RedisCache {
    client: RedisClient,
}

#[cfg(feature = "redis")]
impl RedisCache {
    pub fn new(client: RedisClient) -> Self {
        Self { client }
    }
}

#[cfg(feature = "redis")]
#[async_trait]
impl Cache for RedisCache {
    async fn get_json<T: DeserializeOwned + Send>(
        &self,
        key: &str,
    ) -> Result<Option<T>, CacheError> {
        let mut conn = self.client.get().await.map_err(CacheError::from)?;
        let raw = conn
            .get::<_, Option<String>>(key)
            .await
            .map_err(CacheError::from)?;
        match raw {
            Some(payload) => serde_json::from_str(&payload)
                .map(Some)
                .map_err(CacheError::from),
            None => Ok(None),
        }
    }

    async fn set_json<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl_secs: Option<u64>,
    ) -> Result<(), CacheError> {
        let mut conn = self.client.get().await.map_err(CacheError::from)?;
        let payload = serde_json::to_string(value)?;
        if let Some(ttl) = ttl_secs {
            conn.set_ex::<_, _, ()>(key, payload, ttl as usize)
                .await
                .map_err(CacheError::from)?;
        } else {
            conn.set::<_, _, ()>(key, payload)
                .await
                .map_err(CacheError::from)?;
        }
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let mut conn = self.client.get().await.map_err(CacheError::from)?;
        conn.del::<_, ()>(key).await.map_err(CacheError::from)?;
        Ok(())
    }
}

/// In-memory cache used for testing and fallback scenarios.
pub struct InMemoryCache {
    entries: Arc<Mutex<HashMap<String, (String, Option<Instant>)>>>,
}

impl InMemoryCache {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl Cache for InMemoryCache {
    async fn get_json<T: DeserializeOwned + Send>(
        &self,
        key: &str,
    ) -> Result<Option<T>, CacheError> {
        let mut entries = self.entries.lock().await;
        if let Some((value, expires)) = entries.get(key) {
            if let Some(deadline) = expires {
                if Instant::now() >= *deadline {
                    entries.remove(key);
                    return Ok(None);
                }
            }
            return serde_json::from_str(value)
                .map(Some)
                .map_err(CacheError::from);
        }
        Ok(None)
    }

    async fn set_json<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl_secs: Option<u64>,
    ) -> Result<(), CacheError> {
        let payload = serde_json::to_string(value)?;
        let expiry = ttl_secs.map(|ttl| Instant::now() + Duration::from_secs(ttl));
        let mut entries = self.entries.lock().await;
        entries.insert(key.to_string(), (payload, expiry));
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let mut entries = self.entries.lock().await;
        entries.remove(key);
        Ok(())
    }
}
