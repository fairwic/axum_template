//! Runner order entity

use crate::order::entity::PayStatus;
use crate::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RunnerOrderStatus {
    PendingPay,
    PendingAccept,
    Processing,
    Delivered,
    Completed,
    Canceled,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunnerOrder {
    pub id: Ulid,
    pub user_id: Ulid,
    pub store_id: Ulid,
    pub status: RunnerOrderStatus,
    pub express_company: String,
    pub pickup_code: String,
    pub delivery_address: String,
    pub receiver_name: String,
    pub receiver_phone: String,
    pub remark: Option<String>,
    pub service_fee: i32,
    pub distance_km: Option<f64>,
    pub amount_payable: i32,
    pub pay_status: PayStatus,
    pub pay_time: Option<DateTime<Utc>>,
    pub cancel_reason: Option<String>,
    pub cancel_time: Option<DateTime<Utc>>,
    pub accept_time: Option<DateTime<Utc>>,
    pub delivered_time: Option<DateTime<Utc>>,
    pub complete_time: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RunnerOrder {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        user_id: Ulid,
        store_id: Ulid,
        express_company: String,
        pickup_code: String,
        delivery_address: String,
        receiver_name: String,
        receiver_phone: String,
        remark: Option<String>,
        service_fee: i32,
        distance_km: Option<f64>,
    ) -> Result<Self, DomainError> {
        if express_company.trim().is_empty() {
            return Err(DomainError::Validation(
                "express_company is required".into(),
            ));
        }
        if pickup_code.trim().is_empty() {
            return Err(DomainError::Validation("pickup_code is required".into()));
        }
        if delivery_address.trim().is_empty() {
            return Err(DomainError::Validation(
                "delivery_address is required".into(),
            ));
        }
        if receiver_name.trim().is_empty() {
            return Err(DomainError::Validation("receiver_name is required".into()));
        }
        if receiver_phone.trim().is_empty() {
            return Err(DomainError::Validation("receiver_phone is required".into()));
        }
        if service_fee < 0 {
            return Err(DomainError::Validation(
                "service_fee must be non-negative".into(),
            ));
        }

        let now = Utc::now();
        Ok(Self {
            id: Ulid::new(),
            user_id,
            store_id,
            status: RunnerOrderStatus::PendingPay,
            express_company,
            pickup_code,
            delivery_address,
            receiver_name,
            receiver_phone,
            remark,
            service_fee,
            distance_km,
            amount_payable: service_fee,
            pay_status: PayStatus::Unpaid,
            pay_time: None,
            cancel_reason: None,
            cancel_time: None,
            accept_time: None,
            delivered_time: None,
            complete_time: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn mark_paid(&mut self) -> Result<(), DomainError> {
        if self.status != RunnerOrderStatus::PendingPay {
            return Err(DomainError::InvalidState(
                "only pending pay runner order can be paid".into(),
            ));
        }
        let now = Utc::now();
        self.status = RunnerOrderStatus::PendingAccept;
        self.pay_status = PayStatus::Paid;
        self.pay_time = Some(now);
        self.updated_at = now;
        Ok(())
    }

    pub fn cancel(&mut self, reason: Option<String>) -> Result<(), DomainError> {
        let refund_required = match self.status {
            RunnerOrderStatus::PendingPay => false,
            RunnerOrderStatus::PendingAccept => true,
            _ => {
                return Err(DomainError::InvalidState(
                    "only pending pay/pending accept runner order can be canceled".into(),
                ))
            }
        };
        let now = Utc::now();
        self.status = RunnerOrderStatus::Canceled;
        if refund_required {
            self.pay_status = PayStatus::Refunded;
        }
        self.cancel_reason = reason;
        self.cancel_time = Some(now);
        self.updated_at = now;
        Ok(())
    }

    pub fn admin_accept(&mut self) -> Result<(), DomainError> {
        if self.status != RunnerOrderStatus::PendingAccept {
            return Err(DomainError::InvalidState(
                "only pending accept runner order can be accepted".into(),
            ));
        }
        let now = Utc::now();
        self.status = RunnerOrderStatus::Processing;
        self.accept_time = Some(now);
        self.updated_at = now;
        Ok(())
    }

    pub fn admin_delivered(&mut self) -> Result<(), DomainError> {
        if self.status != RunnerOrderStatus::Processing {
            return Err(DomainError::InvalidState(
                "only processing runner order can be delivered".into(),
            ));
        }
        let now = Utc::now();
        self.status = RunnerOrderStatus::Delivered;
        self.delivered_time = Some(now);
        self.updated_at = now;
        Ok(())
    }

    pub fn admin_complete(&mut self) -> Result<(), DomainError> {
        if self.status != RunnerOrderStatus::Delivered {
            return Err(DomainError::InvalidState(
                "only delivered runner order can be completed".into(),
            ));
        }
        let now = Utc::now();
        self.status = RunnerOrderStatus::Completed;
        self.complete_time = Some(now);
        self.updated_at = now;
        Ok(())
    }
}
