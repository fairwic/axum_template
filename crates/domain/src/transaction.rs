//! Transaction abstractions for cross-repository consistency.

use async_trait::async_trait;
use axum_core_kernel::AppResult;
use ulid::Ulid;

use crate::{GoodsOrder, RunnerOrder};

#[async_trait]
pub trait OrderUnitOfWork: Send {
    async fn try_lock_product_stock(&mut self, product_id: Ulid, qty: i32) -> AppResult<bool>;
    async fn release_product_stock(&mut self, product_id: Ulid, qty: i32) -> AppResult<()>;
    async fn create_goods_order(&mut self, order: &GoodsOrder) -> AppResult<GoodsOrder>;
    async fn update_goods_order(&mut self, order: &GoodsOrder) -> AppResult<GoodsOrder>;
    async fn create_runner_order(&mut self, order: &RunnerOrder) -> AppResult<RunnerOrder>;
    async fn update_runner_order(&mut self, order: &RunnerOrder) -> AppResult<RunnerOrder>;
    async fn commit(self: Box<Self>) -> AppResult<()>;
    async fn rollback(self: Box<Self>) -> AppResult<()>;
}

#[async_trait]
pub trait TransactionManager: Send + Sync {
    async fn begin_order_uow(&self) -> AppResult<Box<dyn OrderUnitOfWork>>;
}
