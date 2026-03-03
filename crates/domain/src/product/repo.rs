//! Product repository trait

use crate::product::entity::Product;
use async_trait::async_trait;
use axum_common::{AppError, AppResult};
use ulid::Ulid;

#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn list_by_category(
        &self,
        store_id: Ulid,
        category_id: Ulid,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<Product>, i64)>;

    async fn search(
        &self,
        store_id: Ulid,
        keyword: &str,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<Product>, i64)>;

    async fn create(&self, product: &Product) -> AppResult<Product>;

    async fn find_by_ids(&self, _store_id: Ulid, _product_ids: &[Ulid]) -> AppResult<Vec<Product>> {
        Err(AppError::Internal("find_by_ids not implemented".into()))
    }

    async fn try_lock_stock(&self, _product_id: Ulid, _qty: i32) -> AppResult<bool> {
        Err(AppError::Internal("try_lock_stock not implemented".into()))
    }

    async fn release_stock(&self, _product_id: Ulid, _qty: i32) -> AppResult<()> {
        Err(AppError::Internal("release_stock not implemented".into()))
    }
}
