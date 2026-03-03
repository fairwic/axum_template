//! Store repository trait

use crate::store::entity::Store;
use async_trait::async_trait;
use axum_core_kernel::AppResult;
use ulid::Ulid;

#[async_trait]
pub trait StoreRepository: Send + Sync {
    async fn list(&self) -> AppResult<Vec<Store>>;
    async fn create(&self, store: &Store) -> AppResult<Store>;
    async fn update(&self, store: &Store) -> AppResult<Store>;
    async fn find_by_id(&self, store_id: Ulid) -> AppResult<Option<Store>>;
}
