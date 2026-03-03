//! Runner order model for persistence

use axum_common::AppError;
use axum_domain::order::entity::PayStatus;
use axum_domain::runner_order::entity::{RunnerOrder, RunnerOrderStatus};
use chrono::{DateTime, Utc};
use ulid::Ulid;

#[derive(Debug, sqlx::FromRow)]
pub struct RunnerOrderModel {
    pub id: String,
    pub user_id: String,
    pub store_id: String,
    pub status: String,
    pub express_company: String,
    pub pickup_code: String,
    pub delivery_address: String,
    pub receiver_name: String,
    pub receiver_phone: String,
    pub remark: Option<String>,
    pub service_fee: i32,
    pub distance_km: Option<f64>,
    pub amount_payable: i32,
    pub pay_status: String,
    pub pay_time: Option<DateTime<Utc>>,
    pub cancel_reason: Option<String>,
    pub cancel_time: Option<DateTime<Utc>>,
    pub accept_time: Option<DateTime<Utc>>,
    pub delivered_time: Option<DateTime<Utc>>,
    pub complete_time: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RunnerOrderModel {
    pub fn from_entity(order: &RunnerOrder) -> Self {
        Self {
            id: order.id.to_string(),
            user_id: order.user_id.to_string(),
            store_id: order.store_id.to_string(),
            status: status_to_string(&order.status).to_string(),
            express_company: order.express_company.clone(),
            pickup_code: order.pickup_code.clone(),
            delivery_address: order.delivery_address.clone(),
            receiver_name: order.receiver_name.clone(),
            receiver_phone: order.receiver_phone.clone(),
            remark: order.remark.clone(),
            service_fee: order.service_fee,
            distance_km: order.distance_km,
            amount_payable: order.amount_payable,
            pay_status: pay_status_to_string(&order.pay_status).to_string(),
            pay_time: order.pay_time,
            cancel_reason: order.cancel_reason.clone(),
            cancel_time: order.cancel_time,
            accept_time: order.accept_time,
            delivered_time: order.delivered_time,
            complete_time: order.complete_time,
            created_at: order.created_at,
            updated_at: order.updated_at,
        }
    }

    pub fn into_entity(self) -> Result<RunnerOrder, AppError> {
        let id = Ulid::from_string(&self.id)
            .map_err(|_| AppError::Internal("invalid runner order id".into()))?;
        let user_id = Ulid::from_string(&self.user_id)
            .map_err(|_| AppError::Internal("invalid runner order user id".into()))?;
        let store_id = Ulid::from_string(&self.store_id)
            .map_err(|_| AppError::Internal("invalid runner order store id".into()))?;
        let status = string_to_status(&self.status)?;
        let pay_status = string_to_pay_status(&self.pay_status)?;

        Ok(RunnerOrder {
            id,
            user_id,
            store_id,
            status,
            express_company: self.express_company,
            pickup_code: self.pickup_code,
            delivery_address: self.delivery_address,
            receiver_name: self.receiver_name,
            receiver_phone: self.receiver_phone,
            remark: self.remark,
            service_fee: self.service_fee,
            distance_km: self.distance_km,
            amount_payable: self.amount_payable,
            pay_status,
            pay_time: self.pay_time,
            cancel_reason: self.cancel_reason,
            cancel_time: self.cancel_time,
            accept_time: self.accept_time,
            delivered_time: self.delivered_time,
            complete_time: self.complete_time,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

fn status_to_string(value: &RunnerOrderStatus) -> &'static str {
    match value {
        RunnerOrderStatus::PendingPay => "PENDING_PAY",
        RunnerOrderStatus::PendingAccept => "PENDING_ACCEPT",
        RunnerOrderStatus::Processing => "PROCESSING",
        RunnerOrderStatus::Delivered => "DELIVERED",
        RunnerOrderStatus::Completed => "COMPLETED",
        RunnerOrderStatus::Canceled => "CANCELED",
        RunnerOrderStatus::Closed => "CLOSED",
    }
}

fn pay_status_to_string(value: &PayStatus) -> &'static str {
    match value {
        PayStatus::Unpaid => "UNPAID",
        PayStatus::Paid => "PAID",
        PayStatus::Refunded => "REFUNDED",
    }
}

fn string_to_status(value: &str) -> Result<RunnerOrderStatus, AppError> {
    match value {
        "PENDING_PAY" => Ok(RunnerOrderStatus::PendingPay),
        "PENDING_ACCEPT" => Ok(RunnerOrderStatus::PendingAccept),
        "PROCESSING" => Ok(RunnerOrderStatus::Processing),
        "DELIVERED" => Ok(RunnerOrderStatus::Delivered),
        "COMPLETED" => Ok(RunnerOrderStatus::Completed),
        "CANCELED" => Ok(RunnerOrderStatus::Canceled),
        "CLOSED" => Ok(RunnerOrderStatus::Closed),
        _ => Err(AppError::Internal("invalid runner order status".into())),
    }
}

fn string_to_pay_status(value: &str) -> Result<PayStatus, AppError> {
    match value {
        "UNPAID" => Ok(PayStatus::Unpaid),
        "PAID" => Ok(PayStatus::Paid),
        "REFUNDED" => Ok(PayStatus::Refunded),
        _ => Err(AppError::Internal("invalid runner order pay_status".into())),
    }
}
