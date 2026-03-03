//! Postgres implementation for StoreRepository

use async_trait::async_trait;
use axum_common::AppResult;
use axum_domain::store::repo::StoreRepository;
use axum_domain::Store;
use sqlx::PgPool;
use ulid::Ulid;

use crate::models::store_model::StoreModel;

pub struct PgStoreRepository {
    pool: PgPool,
}

impl PgStoreRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StoreRepository for PgStoreRepository {
    async fn list(&self) -> AppResult<Vec<Store>> {
        let rows = sqlx::query_as!(
            StoreModel,
            r#"
            SELECT id, name, address, lat, lng, phone, business_hours, status,
                   delivery_radius_km, delivery_fee_base, delivery_fee_per_km, runner_service_fee,
                   created_at, updated_at
            FROM stores
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut stores = Vec::with_capacity(rows.len());
        for model in rows {
            stores.push(model.into_entity()?);
        }
        Ok(stores)
    }

    async fn create(&self, store: &Store) -> AppResult<Store> {
        let model = StoreModel::from_entity(store);
        let row = sqlx::query_as!(
            StoreModel,
            r#"
            INSERT INTO stores (id, name, address, lat, lng, phone, business_hours, status,
                                delivery_radius_km, delivery_fee_base, delivery_fee_per_km, runner_service_fee,
                                created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
            RETURNING id, name, address, lat, lng, phone, business_hours, status,
                      delivery_radius_km, delivery_fee_base, delivery_fee_per_km, runner_service_fee,
                      created_at, updated_at
            "#,
            model.id,
            model.name,
            model.address,
            model.lat,
            model.lng,
            model.phone,
            model.business_hours,
            model.status,
            model.delivery_radius_km,
            model.delivery_fee_base,
            model.delivery_fee_per_km,
            model.runner_service_fee,
            model.created_at,
            model.updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        row.into_entity()
    }

    async fn find_by_id(&self, store_id: Ulid) -> AppResult<Option<Store>> {
        let row = sqlx::query_as!(
            StoreModel,
            r#"
            SELECT id, name, address, lat, lng, phone, business_hours, status,
                   delivery_radius_km, delivery_fee_base, delivery_fee_per_km, runner_service_fee,
                   created_at, updated_at
            FROM stores
            WHERE id = $1
            "#,
            store_id.to_string(),
        )
        .fetch_optional(&self.pool)
        .await?;

        row.map(StoreModel::into_entity).transpose()
    }
}
