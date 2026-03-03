//! Postgres implementation for ProductRepository

use axum_common::AppResult;
use axum_domain::product::repo::ProductRepository;
use axum_domain::Product;
use async_trait::async_trait;
use sqlx::PgPool;
use ulid::Ulid;

use crate::models::product_model::ProductModel;

pub struct PgProductRepository {
    pool: PgPool,
}

impl PgProductRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductRepository for PgProductRepository {
    async fn list_by_category(
        &self,
        store_id: Ulid,
        category_id: Ulid,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<Product>, i64)> {
        let total = sqlx::query_scalar!(
            r#"SELECT COUNT(*)::bigint as "count!" FROM products WHERE store_id = $1 AND category_id = $2"#,
            store_id.to_string(),
            category_id.to_string()
        )
        .fetch_one(&self.pool)
        .await?;

        let offset = (page - 1) * page_size;
        let rows = sqlx::query_as!(
            ProductModel,
            r#"
            SELECT id, store_id, category_id, title, subtitle, cover_image, images, price,
                   original_price, stock, status, tags, created_at, updated_at
            FROM products
            WHERE store_id = $1 AND category_id = $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            store_id.to_string(),
            category_id.to_string(),
            page_size,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let mut products = Vec::with_capacity(rows.len());
        for model in rows {
            products.push(model.into_entity()?);
        }
        Ok((products, total))
    }

    async fn search(
        &self,
        store_id: Ulid,
        keyword: &str,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<Product>, i64)> {
        let like_keyword = format!("%{}%", keyword);
        let total = sqlx::query_scalar!(
            r#"SELECT COUNT(*)::bigint as "count!" FROM products WHERE store_id = $1 AND (title ILIKE $2 OR subtitle ILIKE $2)"#,
            store_id.to_string(),
            like_keyword
        )
        .fetch_one(&self.pool)
        .await?;

        let offset = (page - 1) * page_size;
        let rows = sqlx::query_as!(
            ProductModel,
            r#"
            SELECT id, store_id, category_id, title, subtitle, cover_image, images, price,
                   original_price, stock, status, tags, created_at, updated_at
            FROM products
            WHERE store_id = $1 AND (title ILIKE $2 OR subtitle ILIKE $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            store_id.to_string(),
            like_keyword,
            page_size,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let mut products = Vec::with_capacity(rows.len());
        for model in rows {
            products.push(model.into_entity()?);
        }
        Ok((products, total))
    }

    async fn create(&self, product: &Product) -> AppResult<Product> {
        let model = ProductModel::from_entity(product);
        let row = sqlx::query_as!(
            ProductModel,
            r#"
            INSERT INTO products (id, store_id, category_id, title, subtitle, cover_image, images, price,
                                  original_price, stock, status, tags, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
            RETURNING id, store_id, category_id, title, subtitle, cover_image, images, price,
                      original_price, stock, status, tags, created_at, updated_at
            "#,
            model.id,
            model.store_id,
            model.category_id,
            model.title,
            model.subtitle,
            model.cover_image,
            model.images,
            model.price,
            model.original_price,
            model.stock,
            model.status,
            model.tags,
            model.created_at,
            model.updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        row.into_entity()
    }
}
