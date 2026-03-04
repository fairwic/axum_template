//! Address model

use axum_core_kernel::AppError;
use axum_domain::Address;
use chrono::{DateTime, Utc};
use ulid::Ulid;

#[derive(Debug, sqlx::FromRow)]
pub struct AddressModel {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub phone: String,
    pub detail: String,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AddressModel {
    pub fn from_entity(address: &Address) -> Self {
        Self {
            id: address.id.to_string(),
            user_id: address.user_id.to_string(),
            name: address.name.clone(),
            phone: address.phone.clone(),
            detail: address.detail.clone(),
            lat: address.lat,
            lng: address.lng,
            is_default: address.is_default,
            created_at: address.created_at,
            updated_at: address.updated_at,
        }
    }

    pub fn into_entity(self) -> Result<Address, AppError> {
        Ok(Address {
            id: Ulid::from_string(&self.id)
                .map_err(|_| AppError::Internal("invalid address id".into()))?,
            user_id: Ulid::from_string(&self.user_id)
                .map_err(|_| AppError::Internal("invalid user id".into()))?,
            name: self.name,
            phone: self.phone,
            detail: self.detail,
            lat: self.lat,
            lng: self.lng,
            is_default: self.is_default,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
