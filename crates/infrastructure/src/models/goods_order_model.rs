//! Goods order model for persistence

use axum_common::AppError;
use axum_domain::order::entity::{DeliveryType, GoodsOrder, GoodsOrderStatus, PayStatus};
use chrono::{DateTime, Utc};
use serde_json::Value;
use ulid::Ulid;

#[derive(Debug, sqlx::FromRow)]
pub struct GoodsOrderModel {
    pub id: String,
    pub user_id: String,
    pub store_id: String,
    pub delivery_type: String,
    pub status: String,
    pub items: Value,
    pub amount_goods: i32,
    pub amount_delivery_fee: i32,
    pub amount_discount: i32,
    pub amount_payable: i32,
    pub distance_km: Option<f64>,
    pub address_snapshot: Option<Value>,
    pub store_snapshot: Option<Value>,
    pub remark: Option<String>,
    pub pay_status: String,
    pub pay_time: Option<DateTime<Utc>>,
    pub cancel_reason: Option<String>,
    pub cancel_time: Option<DateTime<Utc>>,
    pub accept_time: Option<DateTime<Utc>>,
    pub complete_time: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl GoodsOrderModel {
    pub fn from_entity(order: &GoodsOrder) -> Result<Self, AppError> {
        let items = serde_json::to_value(&order.items)?;
        Ok(Self {
            id: order.id.to_string(),
            user_id: order.user_id.to_string(),
            store_id: order.store_id.to_string(),
            delivery_type: delivery_type_to_string(&order.delivery_type).to_string(),
            status: status_to_string(&order.status).to_string(),
            items,
            amount_goods: order.amount_goods,
            amount_delivery_fee: order.amount_delivery_fee,
            amount_discount: order.amount_discount,
            amount_payable: order.amount_payable,
            distance_km: order.distance_km,
            address_snapshot: order.address_snapshot.clone(),
            store_snapshot: order.store_snapshot.clone(),
            remark: order.remark.clone(),
            pay_status: pay_status_to_string(&order.pay_status).to_string(),
            pay_time: order.pay_time,
            cancel_reason: order.cancel_reason.clone(),
            cancel_time: order.cancel_time,
            accept_time: order.accept_time,
            complete_time: order.complete_time,
            created_at: order.created_at,
            updated_at: order.updated_at,
        })
    }

    pub fn into_entity(self) -> Result<GoodsOrder, AppError> {
        let id = Ulid::from_string(&self.id)
            .map_err(|_| AppError::Internal("invalid goods order id".into()))?;
        let user_id = Ulid::from_string(&self.user_id)
            .map_err(|_| AppError::Internal("invalid goods order user id".into()))?;
        let store_id = Ulid::from_string(&self.store_id)
            .map_err(|_| AppError::Internal("invalid goods order store id".into()))?;
        let delivery_type = string_to_delivery_type(&self.delivery_type)?;
        let status = string_to_status(&self.status)?;
        let pay_status = string_to_pay_status(&self.pay_status)?;
        let items = serde_json::from_value(self.items)
            .map_err(|_| AppError::Internal("invalid goods order items json".into()))?;

        Ok(GoodsOrder {
            id,
            user_id,
            store_id,
            delivery_type,
            status,
            items,
            amount_goods: self.amount_goods,
            amount_delivery_fee: self.amount_delivery_fee,
            amount_discount: self.amount_discount,
            amount_payable: self.amount_payable,
            distance_km: self.distance_km,
            address_snapshot: self.address_snapshot,
            store_snapshot: self.store_snapshot,
            remark: self.remark,
            pay_status,
            pay_time: self.pay_time,
            cancel_reason: self.cancel_reason,
            cancel_time: self.cancel_time,
            accept_time: self.accept_time,
            complete_time: self.complete_time,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

fn delivery_type_to_string(value: &DeliveryType) -> &'static str {
    match value {
        DeliveryType::Delivery => "DELIVERY",
        DeliveryType::Pickup => "PICKUP",
    }
}

fn status_to_string(value: &GoodsOrderStatus) -> &'static str {
    match value {
        GoodsOrderStatus::PendingPay => "PENDING_PAY",
        GoodsOrderStatus::PendingAccept => "PENDING_ACCEPT",
        GoodsOrderStatus::Accepted => "ACCEPTED",
        GoodsOrderStatus::Delivering => "DELIVERING",
        GoodsOrderStatus::WaitingPickup => "WAITING_PICKUP",
        GoodsOrderStatus::Completed => "COMPLETED",
        GoodsOrderStatus::Canceled => "CANCELED",
        GoodsOrderStatus::Closed => "CLOSED",
    }
}

fn pay_status_to_string(value: &PayStatus) -> &'static str {
    match value {
        PayStatus::Unpaid => "UNPAID",
        PayStatus::Paid => "PAID",
        PayStatus::Refunded => "REFUNDED",
    }
}

fn string_to_delivery_type(value: &str) -> Result<DeliveryType, AppError> {
    match value {
        "DELIVERY" => Ok(DeliveryType::Delivery),
        "PICKUP" => Ok(DeliveryType::Pickup),
        _ => Err(AppError::Internal(
            "invalid goods order delivery_type".into(),
        )),
    }
}

fn string_to_status(value: &str) -> Result<GoodsOrderStatus, AppError> {
    match value {
        "PENDING_PAY" => Ok(GoodsOrderStatus::PendingPay),
        "PENDING_ACCEPT" => Ok(GoodsOrderStatus::PendingAccept),
        "ACCEPTED" => Ok(GoodsOrderStatus::Accepted),
        "DELIVERING" => Ok(GoodsOrderStatus::Delivering),
        "WAITING_PICKUP" => Ok(GoodsOrderStatus::WaitingPickup),
        "COMPLETED" => Ok(GoodsOrderStatus::Completed),
        "CANCELED" => Ok(GoodsOrderStatus::Canceled),
        "CLOSED" => Ok(GoodsOrderStatus::Closed),
        _ => Err(AppError::Internal("invalid goods order status".into())),
    }
}

fn string_to_pay_status(value: &str) -> Result<PayStatus, AppError> {
    match value {
        "UNPAID" => Ok(PayStatus::Unpaid),
        "PAID" => Ok(PayStatus::Paid),
        "REFUNDED" => Ok(PayStatus::Refunded),
        _ => Err(AppError::Internal("invalid goods order pay_status".into())),
    }
}
