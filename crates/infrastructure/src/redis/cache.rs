//! Redis cache service

use async_trait::async_trait;
use axum_common::{AppError, AppResult};
use axum_domain::CacheService;
use fred::{prelude::*, types::ConnectHandle};

pub struct RedisCacheService {
    pool: RedisPool,
    _connection_task: ConnectHandle,
}

impl RedisCacheService {
    pub async fn new(url: &str, max_connections: usize) -> AppResult<Self> {
        let config = RedisConfig::from_url(url)
            .map_err(|err| AppError::Internal(format!("invalid redis url: {err}")))?;
        let builder = Builder::from_config(config);
        let pool = builder
            .build_pool(max_connections)
            .map_err(|err| AppError::Internal(format!("redis pool build error: {err}")))?;
        let connection_task = pool
            .init()
            .await
            .map_err(|err| AppError::Internal(format!("redis init error: {err}")))?;

        Ok(Self {
            pool,
            _connection_task: connection_task,
        })
    }

    fn map_error(err: RedisError) -> AppError {
        AppError::Internal(format!("redis error: {err}"))
    }
}

#[async_trait]
impl CacheService for RedisCacheService {
    async fn get_string(&self, key: &str) -> AppResult<Option<String>> {
        self.pool
            .get::<Option<String>, _>(key)
            .await
            .map_err(Self::map_error)
    }

    async fn set_string(&self, key: &str, value: &str, ttl_secs: u64) -> AppResult<()> {
        let expiration = Expiration::EX(ttl_secs as i64);
        self.pool
            .set::<(), _, _>(key, value, Some(expiration), None, false)
            .await
            .map_err(Self::map_error)
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        let _: i64 = self
            .pool
            .del::<i64, _>(key)
            .await
            .map_err(Self::map_error)?;
        Ok(())
    }
}
