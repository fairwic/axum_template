//! Postgres implementation for AddressRepository

use async_trait::async_trait;
use axum_core_kernel::AppResult;
use axum_domain::address::repo::AddressRepository;
use axum_domain::Address;
use axum_infra_common::map_sqlx_error;
use sqlx::PgPool;
use ulid::Ulid;

use crate::models::address_model::AddressModel;

pub struct PgAddressRepository {
    pool: PgPool,
}

impl PgAddressRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AddressRepository for PgAddressRepository {
    async fn list_by_user(&self, user_id: Ulid) -> AppResult<Vec<Address>> {
        let rows = sqlx::query_as!(
            AddressModel,
            r#"
            SELECT id, user_id, name, phone, detail, lat, lng, is_default, created_at, updated_at
            FROM addresses
            WHERE user_id = $1
            ORDER BY is_default DESC, updated_at DESC
            "#,
            user_id.to_string(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            items.push(row.into_entity()?);
        }
        Ok(items)
    }

    async fn find_by_id(&self, user_id: Ulid, address_id: Ulid) -> AppResult<Option<Address>> {
        let row = sqlx::query_as!(
            AddressModel,
            r#"
            SELECT id, user_id, name, phone, detail, lat, lng, is_default, created_at, updated_at
            FROM addresses
            WHERE id = $1 AND user_id = $2
            "#,
            address_id.to_string(),
            user_id.to_string(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(AddressModel::into_entity).transpose()
    }

    async fn create(&self, address: &Address) -> AppResult<Address> {
        let model = AddressModel::from_entity(address);
        let row = sqlx::query_as!(
            AddressModel,
            r#"
            INSERT INTO addresses (id, user_id, name, phone, detail, lat, lng, is_default, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
            RETURNING id, user_id, name, phone, detail, lat, lng, is_default, created_at, updated_at
            "#,
            model.id,
            model.user_id,
            model.name,
            model.phone,
            model.detail,
            model.lat,
            model.lng,
            model.is_default,
            model.created_at,
            model.updated_at,
        )
        .fetch_one(&self.pool)
        .await.map_err(map_sqlx_error)?;

        row.into_entity()
    }

    async fn update(&self, address: &Address) -> AppResult<Address> {
        let model = AddressModel::from_entity(address);
        let row = sqlx::query_as!(
            AddressModel,
            r#"
            UPDATE addresses
            SET name = $3,
                phone = $4,
                detail = $5,
                lat = $6,
                lng = $7,
                is_default = $8,
                updated_at = $9
            WHERE id = $1 AND user_id = $2
            RETURNING id, user_id, name, phone, detail, lat, lng, is_default, created_at, updated_at
            "#,
            model.id,
            model.user_id,
            model.name,
            model.phone,
            model.detail,
            model.lat,
            model.lng,
            model.is_default,
            model.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.into_entity()
    }

    async fn delete(&self, user_id: Ulid, address_id: Ulid) -> AppResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM addresses
            WHERE id = $1 AND user_id = $2
            "#,
            address_id.to_string(),
            user_id.to_string(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}
