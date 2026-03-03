//! Goods order entity

use crate::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeliveryType {
    Delivery,
    Pickup,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GoodsOrderStatus {
    PendingPay,
    PendingAccept,
    Accepted,
    Delivering,
    WaitingPickup,
    Completed,
    Canceled,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PayStatus {
    Unpaid,
    Paid,
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoodsOrderItem {
    pub product_id: Ulid,
    pub title_snapshot: String,
    pub price_snapshot: i32,
    pub qty: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GoodsOrder {
    pub id: Ulid,
    pub user_id: Ulid,
    pub store_id: Ulid,
    pub delivery_type: DeliveryType,
    pub status: GoodsOrderStatus,
    pub items: Vec<GoodsOrderItem>,
    pub amount_goods: i32,
    pub amount_delivery_fee: i32,
    pub amount_discount: i32,
    pub amount_payable: i32,
    pub distance_km: Option<f64>,
    pub address_snapshot: Option<serde_json::Value>,
    pub store_snapshot: Option<serde_json::Value>,
    pub remark: Option<String>,
    pub pay_status: PayStatus,
    pub pay_time: Option<DateTime<Utc>>,
    pub cancel_reason: Option<String>,
    pub cancel_time: Option<DateTime<Utc>>,
    pub accept_time: Option<DateTime<Utc>>,
    pub complete_time: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl GoodsOrder {
    pub fn new(
        user_id: Ulid,
        store_id: Ulid,
        delivery_type: DeliveryType,
        items: Vec<GoodsOrderItem>,
        delivery_fee: i32,
        distance_km: Option<f64>,
        address_snapshot: Option<serde_json::Value>,
        store_snapshot: Option<serde_json::Value>,
        remark: Option<String>,
    ) -> Result<Self, DomainError> {
        if items.is_empty() {
            return Err(DomainError::Validation("order items required".into()));
        }

        for item in &items {
            if item.qty <= 0 {
                return Err(DomainError::Validation(
                    "item qty must be greater than 0".into(),
                ));
            }
            if item.price_snapshot < 0 {
                return Err(DomainError::Validation(
                    "item price must be non-negative".into(),
                ));
            }
            if item.title_snapshot.trim().is_empty() {
                return Err(DomainError::Validation("item title required".into()));
            }
        }

        if delivery_fee < 0 {
            return Err(DomainError::Validation(
                "delivery fee must be non-negative".into(),
            ));
        }

        match delivery_type {
            DeliveryType::Delivery => {
                if address_snapshot.is_none() {
                    return Err(DomainError::Validation(
                        "address snapshot required for delivery".into(),
                    ));
                }
                if distance_km.is_none() {
                    return Err(DomainError::Validation(
                        "distance_km required for delivery".into(),
                    ));
                }
            }
            DeliveryType::Pickup => {
                if store_snapshot.is_none() {
                    return Err(DomainError::Validation(
                        "store snapshot required for pickup".into(),
                    ));
                }
            }
        }

        let amount_goods = items
            .iter()
            .map(|item| item.price_snapshot * item.qty)
            .sum::<i32>();
        let amount_payable = amount_goods + delivery_fee;
        let now = Utc::now();

        Ok(Self {
            id: Ulid::new(),
            user_id,
            store_id,
            delivery_type,
            status: GoodsOrderStatus::PendingPay,
            items,
            amount_goods,
            amount_delivery_fee: delivery_fee,
            amount_discount: 0,
            amount_payable,
            distance_km,
            address_snapshot,
            store_snapshot,
            remark,
            pay_status: PayStatus::Unpaid,
            pay_time: None,
            cancel_reason: None,
            cancel_time: None,
            accept_time: None,
            complete_time: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn mark_paid(&mut self) -> Result<(), DomainError> {
        if self.status != GoodsOrderStatus::PendingPay {
            return Err(DomainError::InvalidState(
                "only pending pay order can be paid".into(),
            ));
        }
        let now = Utc::now();
        self.status = GoodsOrderStatus::PendingAccept;
        self.pay_status = PayStatus::Paid;
        self.pay_time = Some(now);
        self.updated_at = now;
        Ok(())
    }

    pub fn cancel(&mut self, reason: Option<String>) -> Result<(), DomainError> {
        if self.status != GoodsOrderStatus::PendingPay {
            return Err(DomainError::InvalidState(
                "only pending pay order can be canceled".into(),
            ));
        }
        let now = Utc::now();
        self.status = GoodsOrderStatus::Canceled;
        self.cancel_reason = reason;
        self.cancel_time = Some(now);
        self.updated_at = now;
        Ok(())
    }

    pub fn admin_accept(&mut self) -> Result<(), DomainError> {
        if self.status != GoodsOrderStatus::PendingAccept {
            return Err(DomainError::InvalidState(
                "only pending accept order can be accepted".into(),
            ));
        }
        let now = Utc::now();
        self.status = GoodsOrderStatus::Accepted;
        self.accept_time = Some(now);
        self.updated_at = now;
        Ok(())
    }

    pub fn admin_dispatch(&mut self) -> Result<(), DomainError> {
        if self.status != GoodsOrderStatus::Accepted {
            return Err(DomainError::InvalidState(
                "only accepted order can be dispatched".into(),
            ));
        }
        let now = Utc::now();
        self.status = match self.delivery_type {
            DeliveryType::Delivery => GoodsOrderStatus::Delivering,
            DeliveryType::Pickup => GoodsOrderStatus::WaitingPickup,
        };
        self.updated_at = now;
        Ok(())
    }

    pub fn admin_complete(&mut self) -> Result<(), DomainError> {
        if self.status != GoodsOrderStatus::Delivering
            && self.status != GoodsOrderStatus::WaitingPickup
        {
            return Err(DomainError::InvalidState(
                "only delivering/waiting pickup order can be completed".into(),
            ));
        }
        let now = Utc::now();
        self.status = GoodsOrderStatus::Completed;
        self.complete_time = Some(now);
        self.updated_at = now;
        Ok(())
    }
}
