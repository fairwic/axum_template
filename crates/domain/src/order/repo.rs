//! Goods order repository trait

use crate::order::entity::GoodsOrder;
use async_trait::async_trait;
use axum_core_kernel::AppResult;
use ulid::Ulid;

#[async_trait]
pub trait GoodsOrderRepository: Send + Sync {
    async fn create(&self, order: &GoodsOrder) -> AppResult<GoodsOrder>;
    async fn update(&self, order: &GoodsOrder) -> AppResult<GoodsOrder>;
    async fn find_by_id(&self, order_id: Ulid) -> AppResult<Option<GoodsOrder>>;
    async fn list_by_user(&self, user_id: Ulid) -> AppResult<Vec<GoodsOrder>>;
    async fn list_by_store(&self, store_id: Ulid) -> AppResult<Vec<GoodsOrder>>;
}
