//! Store model for persistence

use axum_core_kernel::AppError;
use axum_domain::store::entity::{Store, StoreStatus};
use chrono::{DateTime, Utc};
use ulid::Ulid;

#[derive(Debug, sqlx::FromRow)]
pub struct StoreModel {
    pub id: String,
    pub name: String,
    pub address: String,
    pub lat: f64,
    pub lng: f64,
    pub phone: String,
    pub business_hours: String,
    pub status: String,
    pub delivery_radius_km: f64,
    pub delivery_fee_base: i32,
    pub delivery_fee_per_km: i32,
    pub runner_service_fee: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl StoreModel {
    pub fn from_entity(store: &Store) -> Self {
        Self {
            id: store.id.to_string(),
            name: store.name.clone(),
            address: store.address.clone(),
            lat: store.lat,
            lng: store.lng,
            phone: store.phone.clone(),
            business_hours: store.business_hours.clone(),
            status: match store.status {
                StoreStatus::Open => "OPEN".to_string(),
                StoreStatus::Closed => "CLOSED".to_string(),
            },
            delivery_radius_km: store.delivery_radius_km,
            delivery_fee_base: store.delivery_fee_base,
            delivery_fee_per_km: store.delivery_fee_per_km,
            runner_service_fee: store.runner_service_fee,
            created_at: store.created_at,
            updated_at: store.updated_at,
        }
    }

    pub fn into_entity(self) -> Result<Store, AppError> {
        let id = Ulid::from_string(&self.id)
            .map_err(|_| AppError::Internal("invalid ulid in database".into()))?;
        let status = match self.status.as_str() {
            "OPEN" => StoreStatus::Open,
            "CLOSED" => StoreStatus::Closed,
            _ => return Err(AppError::Internal("invalid store status".into())),
        };
        Ok(Store {
            id,
            name: self.name,
            address: self.address,
            lat: self.lat,
            lng: self.lng,
            phone: self.phone,
            business_hours: self.business_hours,
            status,
            delivery_radius_km: self.delivery_radius_km,
            delivery_fee_base: self.delivery_fee_base,
            delivery_fee_per_km: self.delivery_fee_per_km,
            runner_service_fee: self.runner_service_fee,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
