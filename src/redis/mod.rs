#![allow(dead_code)]

#[cfg(feature = "redis")]
pub mod redis_cache;
#[cfg(feature = "redis")]
pub mod redis_client;
#[cfg(feature = "redis")]
pub mod redis_lock;
#[cfg(feature = "redis")]
pub mod redis_module;

#[cfg(feature = "redis")]
pub use redis_module::RedisModule;

#[cfg(not(feature = "redis"))]
pub(crate) struct RedisPlaceholder;
