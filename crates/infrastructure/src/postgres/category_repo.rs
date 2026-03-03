//! Postgres implementation for CategoryRepository

use async_trait::async_trait;
use axum_common::AppResult;
use axum_domain::category::repo::CategoryRepository;
use axum_domain::Category;
use sqlx::PgPool;
use ulid::Ulid;

use crate::models::category_model::CategoryModel;

pub struct PgCategoryRepository {
    pool: PgPool,
}

impl PgCategoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CategoryRepository for PgCategoryRepository {
    async fn list_by_store(&self, store_id: Ulid) -> AppResult<Vec<Category>> {
        let rows = sqlx::query_as!(
            CategoryModel,
            r#"
            SELECT id, store_id, name, sort_order, status, created_at, updated_at
            FROM categories
            WHERE store_id = $1
            ORDER BY sort_order ASC, created_at DESC
            "#,
            store_id.to_string()
        )
        .fetch_all(&self.pool)
        .await?;

        let mut categories = Vec::with_capacity(rows.len());
        for model in rows {
            categories.push(model.into_entity()?);
        }
        Ok(categories)
    }

    async fn create(&self, category: &Category) -> AppResult<Category> {
        let model = CategoryModel::from_entity(category);
        let row = sqlx::query_as!(
            CategoryModel,
            r#"
            INSERT INTO categories (id, store_id, name, sort_order, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, store_id, name, sort_order, status, created_at, updated_at
            "#,
            model.id,
            model.store_id,
            model.name,
            model.sort_order,
            model.status,
            model.created_at,
            model.updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        row.into_entity()
    }
}
