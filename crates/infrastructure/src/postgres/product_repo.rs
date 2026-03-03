//! Postgres implementation for ProductRepository

use async_trait::async_trait;
use axum_common::{AppError, AppResult};
use axum_domain::product::repo::ProductRepository;
use axum_domain::Product;
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
        .await.map_err(AppError::database)?;

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
        .await
        .map_err(AppError::database)?;

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
        .await.map_err(AppError::database)?;

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
        .await
        .map_err(AppError::database)?;

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
        .await.map_err(AppError::database)?;

        row.into_entity()
    }

    async fn update(&self, product: &Product) -> AppResult<Product> {
        let model = ProductModel::from_entity(product);
        let row = sqlx::query_as!(
            ProductModel,
            r#"
            UPDATE products
            SET category_id = $2,
                title = $3,
                subtitle = $4,
                cover_image = $5,
                images = $6,
                price = $7,
                original_price = $8,
                stock = $9,
                status = $10,
                tags = $11,
                updated_at = $12
            WHERE id = $1
            RETURNING id, store_id, category_id, title, subtitle, cover_image, images, price,
                      original_price, stock, status, tags, created_at, updated_at
            "#,
            model.id,
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
            model.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(AppError::database)?;

        row.into_entity()
    }

    async fn find_by_id(&self, store_id: Ulid, product_id: Ulid) -> AppResult<Option<Product>> {
        let row = sqlx::query_as!(
            ProductModel,
            r#"
            SELECT id, store_id, category_id, title, subtitle, cover_image, images, price,
                   original_price, stock, status, tags, created_at, updated_at
            FROM products
            WHERE store_id = $1 AND id = $2
            "#,
            store_id.to_string(),
            product_id.to_string()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::database)?;

        row.map(ProductModel::into_entity).transpose()
    }

    async fn find_by_ids(&self, store_id: Ulid, product_ids: &[Ulid]) -> AppResult<Vec<Product>> {
        if product_ids.is_empty() {
            return Ok(vec![]);
        }
        let ids: Vec<String> = product_ids.iter().map(|item| item.to_string()).collect();
        let rows = sqlx::query_as!(
            ProductModel,
            r#"
            SELECT id, store_id, category_id, title, subtitle, cover_image, images, price,
                   original_price, stock, status, tags, created_at, updated_at
            FROM products
            WHERE store_id = $1 AND id = ANY($2::varchar[])
            "#,
            store_id.to_string(),
            &ids
        )
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::database)?;

        let mut products = Vec::with_capacity(rows.len());
        for model in rows {
            products.push(model.into_entity()?);
        }
        Ok(products)
    }

    async fn try_lock_stock(&self, product_id: Ulid, qty: i32) -> AppResult<bool> {
        let row = sqlx::query!(
            r#"
            UPDATE products
            SET stock = stock - $2, updated_at = NOW()
            WHERE id = $1 AND stock >= $2 AND status = 'ON'
            RETURNING id
            "#,
            product_id.to_string(),
            qty
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(AppError::database)?;
        Ok(row.is_some())
    }

    async fn release_stock(&self, product_id: Ulid, qty: i32) -> AppResult<()> {
        sqlx::query!(
            r#"
            UPDATE products
            SET stock = stock + $2, updated_at = NOW()
            WHERE id = $1
            "#,
            product_id.to_string(),
            qty
        )
        .execute(&self.pool)
        .await
        .map_err(AppError::database)?;
        Ok(())
    }
}
