//! Cart repository trait

use crate::cart::entity::Cart;
use async_trait::async_trait;
use axum_common::AppResult;
use ulid::Ulid;

#[async_trait]
pub trait CartRepository: Send + Sync {
    async fn get_cart(&self, user_id: Ulid, store_id: Ulid) -> AppResult<Option<Cart>>;
    async fn create_cart(&self, user_id: Ulid, store_id: Ulid) -> AppResult<Cart>;
    async fn upsert_item(
        &self,
        user_id: Ulid,
        store_id: Ulid,
        product_id: Ulid,
        qty: i32,
        price_snapshot: i32,
    ) -> AppResult<()>;
    async fn remove_item(&self, user_id: Ulid, store_id: Ulid, product_id: Ulid) -> AppResult<()>;
    async fn clear(&self, user_id: Ulid, store_id: Ulid) -> AppResult<()>;
}
