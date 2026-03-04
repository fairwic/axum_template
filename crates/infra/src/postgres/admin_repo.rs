//! Postgres implementation for AdminRepository

use async_trait::async_trait;
use axum_infra_common::map_sqlx_error;
use axum_core_kernel::AppResult;
use axum_domain::admin::repo::AdminRepository;
use axum_domain::Admin;
use sqlx::PgPool;

use crate::models::admin_model::AdminModel;

pub struct PgAdminRepository {
    pool: PgPool,
}

impl PgAdminRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AdminRepository for PgAdminRepository {
    async fn find_by_phone(&self, phone: &str) -> AppResult<Option<Admin>> {
        let row = sqlx::query_as!(
            AdminModel,
            r#"
            SELECT id, phone, password_hash, role, store_id, created_at, updated_at
            FROM admins
            WHERE phone = $1
            "#,
            phone
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        match row {
            Some(model) => Ok(Some(model.into_entity()?)),
            None => Ok(None),
        }
    }

    async fn create(&self, admin: &Admin) -> AppResult<Admin> {
        let model = AdminModel::from_entity(admin);
        let row = sqlx::query_as!(
            AdminModel,
            r#"
            INSERT INTO admins (id, phone, password_hash, role, store_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, phone, password_hash, role, store_id, created_at, updated_at
            "#,
            model.id,
            model.phone,
            model.password_hash,
            model.role,
            model.store_id,
            model.created_at,
            model.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.into_entity()
    }
}
