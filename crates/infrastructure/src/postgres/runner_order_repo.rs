//! Postgres implementation for RunnerOrderRepository

use axum_common::AppResult;
use axum_domain::runner_order::repo::RunnerOrderRepository;
use axum_domain::RunnerOrder;
use async_trait::async_trait;
use sqlx::PgPool;
use ulid::Ulid;

use crate::models::runner_order_model::RunnerOrderModel;

pub struct PgRunnerOrderRepository {
    pool: PgPool,
}

impl PgRunnerOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RunnerOrderRepository for PgRunnerOrderRepository {
    async fn create(&self, order: &RunnerOrder) -> AppResult<RunnerOrder> {
        let model = RunnerOrderModel::from_entity(order);
        let row = sqlx::query_as!(
            RunnerOrderModel,
            r#"
            INSERT INTO runner_orders (
                id, user_id, store_id, status,
                express_company, pickup_code, delivery_address, receiver_name, receiver_phone,
                remark, service_fee, distance_km, amount_payable,
                pay_status, pay_time, cancel_reason, cancel_time,
                accept_time, delivered_time, complete_time, created_at, updated_at
            ) VALUES (
                $1,$2,$3,$4,
                $5,$6,$7,$8,$9,
                $10,$11,$12,$13,
                $14,$15,$16,$17,
                $18,$19,$20,$21,$22
            )
            RETURNING
                id, user_id, store_id, status,
                express_company, pickup_code, delivery_address, receiver_name, receiver_phone,
                remark, service_fee, distance_km, amount_payable,
                pay_status, pay_time, cancel_reason, cancel_time,
                accept_time, delivered_time, complete_time, created_at, updated_at
            "#,
            model.id,
            model.user_id,
            model.store_id,
            model.status,
            model.express_company,
            model.pickup_code,
            model.delivery_address,
            model.receiver_name,
            model.receiver_phone,
            model.remark,
            model.service_fee,
            model.distance_km,
            model.amount_payable,
            model.pay_status,
            model.pay_time,
            model.cancel_reason,
            model.cancel_time,
            model.accept_time,
            model.delivered_time,
            model.complete_time,
            model.created_at,
            model.updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        row.into_entity()
    }

    async fn update(&self, order: &RunnerOrder) -> AppResult<RunnerOrder> {
        let model = RunnerOrderModel::from_entity(order);
        let row = sqlx::query_as!(
            RunnerOrderModel,
            r#"
            UPDATE runner_orders
            SET
                status = $2,
                remark = $3,
                service_fee = $4,
                distance_km = $5,
                amount_payable = $6,
                pay_status = $7,
                pay_time = $8,
                cancel_reason = $9,
                cancel_time = $10,
                accept_time = $11,
                delivered_time = $12,
                complete_time = $13,
                updated_at = $14
            WHERE id = $1
            RETURNING
                id, user_id, store_id, status,
                express_company, pickup_code, delivery_address, receiver_name, receiver_phone,
                remark, service_fee, distance_km, amount_payable,
                pay_status, pay_time, cancel_reason, cancel_time,
                accept_time, delivered_time, complete_time, created_at, updated_at
            "#,
            model.id,
            model.status,
            model.remark,
            model.service_fee,
            model.distance_km,
            model.amount_payable,
            model.pay_status,
            model.pay_time,
            model.cancel_reason,
            model.cancel_time,
            model.accept_time,
            model.delivered_time,
            model.complete_time,
            model.updated_at,
        )
        .fetch_one(&self.pool)
        .await?;

        row.into_entity()
    }

    async fn find_by_id(&self, order_id: Ulid) -> AppResult<Option<RunnerOrder>> {
        let row = sqlx::query_as!(
            RunnerOrderModel,
            r#"
            SELECT
                id, user_id, store_id, status,
                express_company, pickup_code, delivery_address, receiver_name, receiver_phone,
                remark, service_fee, distance_km, amount_payable,
                pay_status, pay_time, cancel_reason, cancel_time,
                accept_time, delivered_time, complete_time, created_at, updated_at
            FROM runner_orders
            WHERE id = $1
            "#,
            order_id.to_string()
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(value) => Ok(Some(value.into_entity()?)),
            None => Ok(None),
        }
    }

    async fn list_by_user(&self, user_id: Ulid) -> AppResult<Vec<RunnerOrder>> {
        let rows = sqlx::query_as!(
            RunnerOrderModel,
            r#"
            SELECT
                id, user_id, store_id, status,
                express_company, pickup_code, delivery_address, receiver_name, receiver_phone,
                remark, service_fee, distance_km, amount_payable,
                pay_status, pay_time, cancel_reason, cancel_time,
                accept_time, delivered_time, complete_time, created_at, updated_at
            FROM runner_orders
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id.to_string()
        )
        .fetch_all(&self.pool)
        .await?;

        let mut orders = Vec::with_capacity(rows.len());
        for row in rows {
            orders.push(row.into_entity()?);
        }
        Ok(orders)
    }

    async fn list_by_store(&self, store_id: Ulid) -> AppResult<Vec<RunnerOrder>> {
        let rows = sqlx::query_as!(
            RunnerOrderModel,
            r#"
            SELECT
                id, user_id, store_id, status,
                express_company, pickup_code, delivery_address, receiver_name, receiver_phone,
                remark, service_fee, distance_km, amount_payable,
                pay_status, pay_time, cancel_reason, cancel_time,
                accept_time, delivered_time, complete_time, created_at, updated_at
            FROM runner_orders
            WHERE store_id = $1
            ORDER BY created_at DESC
            "#,
            store_id.to_string()
        )
        .fetch_all(&self.pool)
        .await?;

        let mut orders = Vec::with_capacity(rows.len());
        for row in rows {
            orders.push(row.into_entity()?);
        }
        Ok(orders)
    }
}
