//! Cart models for persistence

use axum_core_kernel::AppError;
use axum_domain::cart::entity::{Cart, CartItem};
use chrono::{DateTime, Utc};
use ulid::Ulid;

#[derive(Debug, sqlx::FromRow)]
pub struct CartModel {
    pub id: String,
    pub user_id: String,
    pub store_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CartItemModel {
    pub cart_id: String,
    pub product_id: String,
    pub qty: i32,
    pub price_snapshot: i32,
}

impl CartModel {
    pub fn into_entity(self, items: Vec<CartItem>) -> Result<Cart, AppError> {
        let id = Ulid::from_string(&self.id)
            .map_err(|_| AppError::Internal("invalid cart id".into()))?;
        let user_id = Ulid::from_string(&self.user_id)
            .map_err(|_| AppError::Internal("invalid user id".into()))?;
        let store_id = Ulid::from_string(&self.store_id)
            .map_err(|_| AppError::Internal("invalid store id".into()))?;
        Ok(Cart {
            id,
            user_id,
            store_id,
            items,
        })
    }
}

impl CartItemModel {
    pub fn into_entity(self) -> Result<CartItem, AppError> {
        let product_id = Ulid::from_string(&self.product_id)
            .map_err(|_| AppError::Internal("invalid product id".into()))?;
        Ok(CartItem {
            product_id,
            qty: self.qty,
            price_snapshot: self.price_snapshot,
        })
    }
}
