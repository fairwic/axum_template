//! Postgres implementation for GoodsOrderRepository

use axum_common::AppResult;
use axum_domain::order::repo::GoodsOrderRepository;
use axum_domain::GoodsOrder;
use async_trait::async_trait;
use sqlx::PgPool;
use ulid::Ulid;

use crate::models::goods_order_model::GoodsOrderModel;

pub struct PgGoodsOrderRepository {
    pool: PgPool,
}

impl PgGoodsOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GoodsOrderRepository for PgGoodsOrderRepository {
    async fn create(&self, order: &GoodsOrder) -> AppResult<GoodsOrder> {
        let model = GoodsOrderModel::from_entity(order)?;
        let row = sqlx::query_as!(
            GoodsOrderModel,
            r#"
            INSERT INTO goods_orders (
                id, user_id, store_id, delivery_type, status, items,
                amount_goods, amount_delivery_fee, amount_discount, amount_payable,
                distance_km, address_snapshot, store_snapshot, remark,
                pay_status, pay_time, cancel_reason, cancel_time,
                accept_time, complete_time, created_at, updated_at
            ) VALUES (
                $1,$2,$3,$4,$5,$6,
                $7,$8,$9,$10,
                $11,$12,$13,$14,
                $15,$16,$17,$18,
                $19,$20,$21,$22
            )
            RETURNING
                id, user_id, store_id, delivery_type, status, items,
                amount_goods, amount_delivery_fee, amount_discount, amount_payable,
                distance_km, address_snapshot, store_snapshot, remark,
                pay_status, pay_time, cancel_reason, cancel_time,
                accept_time, complete_time, created_at, updated_at
            "#,
            model.id,
            model.user_id,
            model.store_id,
            model.delivery_type,
            model.status,
            model.items,
            model.amount_goods,
            model.amount_delivery_fee,
            model.amount_discount,
            model.amount_payable,
            model.distance_km,
            model.address_snapshot,
            model.store_snapshot,
            model.remark,
            model.pay_status,
            model.pay_time,
            model.cancel_reason,
            model.cancel_time,
            model.accept_time,
            model.complete_time,
            model.created_at,
            model.updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        row.into_entity()
    }

    async fn update(&self, order: &GoodsOrder) -> AppResult<GoodsOrder> {
        let model = GoodsOrderModel::from_entity(order)?;
        let row = sqlx::query_as!(
            GoodsOrderModel,
            r#"
            UPDATE goods_orders
            SET
                status = $2,
                amount_goods = $3,
                amount_delivery_fee = $4,
                amount_discount = $5,
                amount_payable = $6,
                distance_km = $7,
                address_snapshot = $8,
                store_snapshot = $9,
                remark = $10,
                pay_status = $11,
                pay_time = $12,
                cancel_reason = $13,
                cancel_time = $14,
                accept_time = $15,
                complete_time = $16,
                updated_at = $17,
                items = $18
            WHERE id = $1
            RETURNING
                id, user_id, store_id, delivery_type, status, items,
                amount_goods, amount_delivery_fee, amount_discount, amount_payable,
                distance_km, address_snapshot, store_snapshot, remark,
                pay_status, pay_time, cancel_reason, cancel_time,
                accept_time, complete_time, created_at, updated_at
            "#,
            model.id,
            model.status,
            model.amount_goods,
            model.amount_delivery_fee,
            model.amount_discount,
            model.amount_payable,
            model.distance_km,
            model.address_snapshot,
            model.store_snapshot,
            model.remark,
            model.pay_status,
            model.pay_time,
            model.cancel_reason,
            model.cancel_time,
            model.accept_time,
            model.complete_time,
            model.updated_at,
            model.items,
        )
        .fetch_one(&self.pool)
        .await?;

        row.into_entity()
    }

    async fn find_by_id(&self, order_id: Ulid) -> AppResult<Option<GoodsOrder>> {
        let row = sqlx::query_as!(
            GoodsOrderModel,
            r#"
            SELECT
                id, user_id, store_id, delivery_type, status, items,
                amount_goods, amount_delivery_fee, amount_discount, amount_payable,
                distance_km, address_snapshot, store_snapshot, remark,
                pay_status, pay_time, cancel_reason, cancel_time,
                accept_time, complete_time, created_at, updated_at
            FROM goods_orders
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

    async fn list_by_user(&self, user_id: Ulid) -> AppResult<Vec<GoodsOrder>> {
        let rows = sqlx::query_as!(
            GoodsOrderModel,
            r#"
            SELECT
                id, user_id, store_id, delivery_type, status, items,
                amount_goods, amount_delivery_fee, amount_discount, amount_payable,
                distance_km, address_snapshot, store_snapshot, remark,
                pay_status, pay_time, cancel_reason, cancel_time,
                accept_time, complete_time, created_at, updated_at
            FROM goods_orders
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

    async fn list_by_store(&self, store_id: Ulid) -> AppResult<Vec<GoodsOrder>> {
        let rows = sqlx::query_as!(
            GoodsOrderModel,
            r#"
            SELECT
                id, user_id, store_id, delivery_type, status, items,
                amount_goods, amount_delivery_fee, amount_discount, amount_payable,
                distance_km, address_snapshot, store_snapshot, remark,
                pay_status, pay_time, cancel_reason, cancel_time,
                accept_time, complete_time, created_at, updated_at
            FROM goods_orders
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
