//! In-memory cache service

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum_common::AppResult;
use axum_domain::CacheService;
use chrono::{DateTime, Duration, Utc};
use tokio::sync::Mutex;

#[derive(Clone, Default)]
pub struct MemoryCacheService {
    inner: Arc<Mutex<HashMap<String, CacheEntry>>>,
}

#[derive(Clone)]
struct CacheEntry {
    value: String,
    expires_at: DateTime<Utc>,
}

impl MemoryCacheService {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl CacheService for MemoryCacheService {
    async fn get_string(&self, key: &str) -> AppResult<Option<String>> {
        let mut guard = self.inner.lock().await;
        match guard.get(key) {
            Some(entry) if entry.expires_at > Utc::now() => Ok(Some(entry.value.clone())),
            Some(_) => {
                guard.remove(key);
                Ok(None)
            }
            None => Ok(None),
        }
    }

    async fn set_string(&self, key: &str, value: &str, ttl_secs: u64) -> AppResult<()> {
        let expires_at = Utc::now() + Duration::seconds(ttl_secs as i64);
        let entry = CacheEntry {
            value: value.to_string(),
            expires_at,
        };

        let mut guard = self.inner.lock().await;
        guard.insert(key.to_string(), entry);
        Ok(())
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        let mut guard = self.inner.lock().await;
        guard.remove(key);
        Ok(())
    }
}
