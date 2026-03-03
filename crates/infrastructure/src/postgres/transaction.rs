//! Postgres transaction manager for order write consistency.

use async_trait::async_trait;
use axum_common::{AppError, AppResult};
use axum_domain::{GoodsOrder, OrderUnitOfWork, RunnerOrder, TransactionManager};
use sqlx::{PgPool, Postgres, Transaction};
use ulid::Ulid;

use crate::models::goods_order_model::GoodsOrderModel;
use crate::models::runner_order_model::RunnerOrderModel;

#[derive(Clone)]
pub struct PgTransactionManager {
    pool: PgPool,
}

impl PgTransactionManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

pub struct PgOrderUnitOfWork {
    tx: Option<Transaction<'static, Postgres>>,
}

impl PgOrderUnitOfWork {
    fn tx_mut(&mut self) -> AppResult<&mut Transaction<'static, Postgres>> {
        self.tx
            .as_mut()
            .ok_or_else(|| AppError::Internal("transaction already closed".into()))
    }
}

#[async_trait]
impl TransactionManager for PgTransactionManager {
    async fn begin_order_uow(&self) -> AppResult<Box<dyn OrderUnitOfWork>> {
        let tx = self.pool.begin().await.map_err(AppError::database)?;
        Ok(Box::new(PgOrderUnitOfWork { tx: Some(tx) }))
    }
}

#[async_trait]
impl OrderUnitOfWork for PgOrderUnitOfWork {
    async fn try_lock_product_stock(&mut self, product_id: Ulid, qty: i32) -> AppResult<bool> {
        let tx = self.tx_mut()?;
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
        .fetch_optional(tx.as_mut())
        .await
        .map_err(AppError::database)?;

        Ok(row.is_some())
    }

    async fn release_product_stock(&mut self, product_id: Ulid, qty: i32) -> AppResult<()> {
        let tx = self.tx_mut()?;
        sqlx::query!(
            r#"
            UPDATE products
            SET stock = stock + $2, updated_at = NOW()
            WHERE id = $1
            "#,
            product_id.to_string(),
            qty
        )
        .execute(tx.as_mut())
        .await
        .map_err(AppError::database)?;

        Ok(())
    }

    async fn create_goods_order(&mut self, order: &GoodsOrder) -> AppResult<GoodsOrder> {
        let model = GoodsOrderModel::from_entity(order)?;
        let tx = self.tx_mut()?;
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
        .fetch_one(tx.as_mut())
        .await
        .map_err(AppError::database)?;

        row.into_entity()
    }

    async fn update_goods_order(&mut self, order: &GoodsOrder) -> AppResult<GoodsOrder> {
        let model = GoodsOrderModel::from_entity(order)?;
        let tx = self.tx_mut()?;
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
        .fetch_one(tx.as_mut())
        .await
        .map_err(AppError::database)?;

        row.into_entity()
    }

    async fn create_runner_order(&mut self, order: &RunnerOrder) -> AppResult<RunnerOrder> {
        let model = RunnerOrderModel::from_entity(order);
        let tx = self.tx_mut()?;
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
        .fetch_one(tx.as_mut())
        .await
        .map_err(AppError::database)?;

        row.into_entity()
    }

    async fn update_runner_order(&mut self, order: &RunnerOrder) -> AppResult<RunnerOrder> {
        let model = RunnerOrderModel::from_entity(order);
        let tx = self.tx_mut()?;
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
        .fetch_one(tx.as_mut())
        .await
        .map_err(AppError::database)?;

        row.into_entity()
    }

    async fn commit(mut self: Box<Self>) -> AppResult<()> {
        let tx = self
            .tx
            .take()
            .ok_or_else(|| AppError::Internal("transaction already closed".into()))?;
        tx.commit().await.map_err(AppError::database)?;
        Ok(())
    }

    async fn rollback(mut self: Box<Self>) -> AppResult<()> {
        let tx = self
            .tx
            .take()
            .ok_or_else(|| AppError::Internal("transaction already closed".into()))?;
        tx.rollback().await.map_err(AppError::database)?;
        Ok(())
    }
}
