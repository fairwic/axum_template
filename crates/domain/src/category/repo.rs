//! Category repository trait

use crate::category::entity::Category;
use async_trait::async_trait;
use axum_core_kernel::AppResult;
use ulid::Ulid;

#[async_trait]
pub trait CategoryRepository: Send + Sync {
    async fn list_by_store(&self, store_id: Ulid) -> AppResult<Vec<Category>>;
    async fn create(&self, category: &Category) -> AppResult<Category>;
    async fn find_by_id(&self, category_id: Ulid) -> AppResult<Option<Category>>;
    async fn update(&self, category: &Category) -> AppResult<Category>;
}
