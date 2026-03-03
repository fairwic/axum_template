//! Store repository trait

use crate::store::entity::Store;
use async_trait::async_trait;
use axum_common::{AppError, AppResult};
use ulid::Ulid;

#[async_trait]
pub trait StoreRepository: Send + Sync {
    async fn list(&self) -> AppResult<Vec<Store>>;
    async fn create(&self, store: &Store) -> AppResult<Store>;
    async fn update(&self, _store: &Store) -> AppResult<Store> {
        Err(AppError::Internal("update not implemented".into()))
    }
    async fn find_by_id(&self, _store_id: Ulid) -> AppResult<Option<Store>> {
        Ok(None)
    }
}
