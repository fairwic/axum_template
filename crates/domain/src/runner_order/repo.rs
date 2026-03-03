//! Runner order repository trait

use crate::runner_order::entity::RunnerOrder;
use async_trait::async_trait;
use axum_common::AppResult;
use ulid::Ulid;

#[async_trait]
pub trait RunnerOrderRepository: Send + Sync {
    async fn create(&self, order: &RunnerOrder) -> AppResult<RunnerOrder>;
    async fn update(&self, order: &RunnerOrder) -> AppResult<RunnerOrder>;
    async fn find_by_id(&self, order_id: Ulid) -> AppResult<Option<RunnerOrder>>;
    async fn list_by_user(&self, user_id: Ulid) -> AppResult<Vec<RunnerOrder>>;
    async fn list_by_store(&self, store_id: Ulid) -> AppResult<Vec<RunnerOrder>>;
}
