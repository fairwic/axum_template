//! Postgres implementation for CartRepository

use async_trait::async_trait;
use axum_common_infra::map_sqlx_error;
use axum_core_kernel::AppResult;
use axum_domain::cart::repo::CartRepository;
use axum_domain::Cart;
use chrono::Utc;
use sqlx::PgPool;
use ulid::Ulid;

use crate::models::cart_model::{CartItemModel, CartModel};

pub struct PgCartRepository {
    pool: PgPool,
}

impl PgCartRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn find_cart_id(&self, user_id: Ulid, store_id: Ulid) -> AppResult<Option<String>> {
        let row = sqlx::query!(
            r#"SELECT id FROM carts WHERE user_id = $1 AND store_id = $2"#,
            user_id.to_string(),
            store_id.to_string()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;
        Ok(row.map(|r| r.id))
    }
}

#[async_trait]
impl CartRepository for PgCartRepository {
    async fn get_cart(&self, user_id: Ulid, store_id: Ulid) -> AppResult<Option<Cart>> {
        let cart_row = sqlx::query_as!(
            CartModel,
            r#"SELECT id, user_id, store_id, created_at, updated_at FROM carts WHERE user_id = $1 AND store_id = $2"#,
            user_id.to_string(),
            store_id.to_string()
        )
        .fetch_optional(&self.pool)
        .await.map_err(map_sqlx_error)?;

        let Some(cart) = cart_row else {
            return Ok(None);
        };

        let items = sqlx::query_as!(
            CartItemModel,
            r#"SELECT cart_id, product_id, qty, price_snapshot FROM cart_items WHERE cart_id = $1"#,
            cart.id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let mut cart_items = Vec::with_capacity(items.len());
        for item in items {
            cart_items.push(item.into_entity()?);
        }

        Ok(Some(cart.into_entity(cart_items)?))
    }

    async fn create_cart(&self, user_id: Ulid, store_id: Ulid) -> AppResult<Cart> {
        let now = Utc::now();
        let cart_id = Ulid::new().to_string();
        let row = sqlx::query_as!(
            CartModel,
            r#"
            INSERT INTO carts (id, user_id, store_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, store_id, created_at, updated_at
            "#,
            cart_id,
            user_id.to_string(),
            store_id.to_string(),
            now,
            now
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into_entity(vec![])?)
    }

    async fn upsert_item(
        &self,
        user_id: Ulid,
        store_id: Ulid,
        product_id: Ulid,
        qty: i32,
        price_snapshot: i32,
    ) -> AppResult<()> {
        let Some(cart_id) = self.find_cart_id(user_id, store_id).await? else {
            return Ok(());
        };

        sqlx::query!(
            r#"
            INSERT INTO cart_items (cart_id, product_id, qty, price_snapshot)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (cart_id, product_id)
            DO UPDATE SET qty = EXCLUDED.qty, price_snapshot = EXCLUDED.price_snapshot
            "#,
            cart_id,
            product_id.to_string(),
            qty,
            price_snapshot
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn remove_item(&self, user_id: Ulid, store_id: Ulid, product_id: Ulid) -> AppResult<()> {
        let Some(cart_id) = self.find_cart_id(user_id, store_id).await? else {
            return Ok(());
        };
        sqlx::query!(
            r#"DELETE FROM cart_items WHERE cart_id = $1 AND product_id = $2"#,
            cart_id,
            product_id.to_string()
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn clear(&self, user_id: Ulid, store_id: Ulid) -> AppResult<()> {
        let Some(cart_id) = self.find_cart_id(user_id, store_id).await? else {
            return Ok(());
        };
        sqlx::query!(r#"DELETE FROM cart_items WHERE cart_id = $1"#, cart_id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }
}
