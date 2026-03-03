//! Store entity

use crate::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StoreStatus {
    Open,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Store {
    pub id: Ulid,
    pub name: String,
    pub address: String,
    pub lat: f64,
    pub lng: f64,
    pub phone: String,
    pub business_hours: String,
    pub status: StoreStatus,
    pub delivery_radius_km: f64,
    pub delivery_fee_base: i32,
    pub delivery_fee_per_km: i32,
    pub runner_service_fee: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Store {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        address: String,
        lat: f64,
        lng: f64,
        phone: String,
        business_hours: String,
        status: StoreStatus,
        delivery_radius_km: f64,
        delivery_fee_base: i32,
        delivery_fee_per_km: i32,
        runner_service_fee: i32,
    ) -> Result<Self, DomainError> {
        Self::validate_name(&name)?;
        Self::validate_address(&address)?;

        let now = Utc::now();
        Ok(Self {
            id: Ulid::new(),
            name,
            address,
            lat,
            lng,
            phone,
            business_hours,
            status,
            delivery_radius_km,
            delivery_fee_base,
            delivery_fee_per_km,
            runner_service_fee,
            created_at: now,
            updated_at: now,
        })
    }

    fn validate_name(name: &str) -> Result<(), DomainError> {
        if name.trim().is_empty() {
            return Err(DomainError::Validation("store name is required".into()));
        }
        Ok(())
    }

    fn validate_address(address: &str) -> Result<(), DomainError> {
        if address.trim().is_empty() {
            return Err(DomainError::Validation("address is required".into()));
        }
        Ok(())
    }
}
