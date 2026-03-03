//! Cache service trait

use async_trait::async_trait;

use axum_core_kernel::AppResult;

#[async_trait]
pub trait CacheService: Send + Sync {
    async fn get_string(&self, key: &str) -> AppResult<Option<String>>;
    async fn set_string(&self, key: &str, value: &str, ttl_secs: u64) -> AppResult<()>;
    async fn delete(&self, key: &str) -> AppResult<()>;
}
