//! Cart service

use std::sync::Arc;

use axum_core_kernel::AppResult;
use axum_domain::cart::entity::Cart;
use axum_domain::cart::repo::CartRepository;
use ulid::Ulid;

#[derive(Clone)]
pub struct CartService {
    repo: Arc<dyn CartRepository>,
}

impl CartService {
    pub fn new(repo: Arc<dyn CartRepository>) -> Self {
        Self { repo }
    }

    pub async fn get_cart(&self, user_id: Ulid, store_id: Ulid) -> AppResult<Cart> {
        if let Some(cart) = self.repo.get_cart(user_id, store_id).await? {
            return Ok(cart);
        }
        self.repo.create_cart(user_id, store_id).await
    }

    pub async fn add_item(
        &self,
        user_id: Ulid,
        store_id: Ulid,
        product_id: Ulid,
        qty: i32,
        price_snapshot: i32,
    ) -> AppResult<()> {
        if self.repo.get_cart(user_id, store_id).await?.is_none() {
            self.repo.create_cart(user_id, store_id).await?;
        }
        self.repo
            .upsert_item(user_id, store_id, product_id, qty, price_snapshot)
            .await
    }

    pub async fn remove_item(
        &self,
        user_id: Ulid,
        store_id: Ulid,
        product_id: Ulid,
    ) -> AppResult<()> {
        self.repo.remove_item(user_id, store_id, product_id).await
    }

    pub async fn clear(&self, user_id: Ulid, store_id: Ulid) -> AppResult<()> {
        self.repo.clear(user_id, store_id).await
    }
}
