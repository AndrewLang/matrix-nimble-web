#[cfg(feature = "redis")]
use deadpool::managed::PoolError;
#[cfg(feature = "redis")]
use deadpool_redis::redis::RedisError;
#[cfg(feature = "redis")]
use deadpool_redis::{Config, Pool, Runtime};

#[cfg(feature = "redis")]
#[derive(Clone)]
pub struct RedisClient {
    pool: Pool,
    pub pool_size: usize,
    pub url: String,
}

#[cfg(feature = "redis")]
impl RedisClient {
    pub fn new(url: &str, pool_size: usize) -> Self {
        let mut config = Config::from_url(url);
        let mut pool_config = config.pool.unwrap_or_default();
        pool_config.max_size = pool_size;
        config.pool = Some(pool_config);
        let pool = config
            .create_pool(Some(Runtime::Tokio1))
            .expect("failed to create redis pool");

        Self {
            pool,
            pool_size,
            url: url.to_string(),
        }
    }

    pub async fn get(&self) -> Result<deadpool_redis::Connection, PoolError<RedisError>> {
        self.pool.get().await
    }
}
