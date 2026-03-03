//! Store repository trait

use crate::store::entity::Store;
use axum_common::AppResult;
use async_trait::async_trait;

#[async_trait]
pub trait StoreRepository: Send + Sync {
    async fn list(&self) -> AppResult<Vec<Store>>;
    async fn create(&self, store: &Store) -> AppResult<Store>;
}
