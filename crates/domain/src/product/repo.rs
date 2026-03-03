//! Product repository trait

use crate::product::entity::Product;
use axum_common::AppResult;
use async_trait::async_trait;
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
}
