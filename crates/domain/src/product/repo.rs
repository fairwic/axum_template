//! Product repository trait

use crate::product::entity::Product;
use async_trait::async_trait;
use axum_core_kernel::AppResult;
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
    async fn update(&self, product: &Product) -> AppResult<Product>;

    async fn find_by_id(&self, store_id: Ulid, product_id: Ulid) -> AppResult<Option<Product>>;

    async fn find_by_ids(&self, store_id: Ulid, product_ids: &[Ulid]) -> AppResult<Vec<Product>>;

    async fn try_lock_stock(&self, product_id: Ulid, qty: i32) -> AppResult<bool>;

    async fn release_stock(&self, product_id: Ulid, qty: i32) -> AppResult<()>;
}
