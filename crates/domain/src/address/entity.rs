//! Address entity

use crate::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Address {
    pub id: Ulid,
    pub user_id: Ulid,
    pub name: String,
    pub phone: String,
    pub detail: String,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Address {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        user_id: Ulid,
        name: String,
        phone: String,
        detail: String,
        lat: Option<f64>,
        lng: Option<f64>,
        is_default: bool,
    ) -> Result<Self, DomainError> {
        Self::validate_name(&name)?;
        Self::validate_phone(&phone)?;
        Self::validate_detail(&detail)?;
        Self::validate_geo(lat, lng)?;

        let now = Utc::now();
        Ok(Self {
            id: Ulid::new(),
            user_id,
            name,
            phone,
            detail,
            lat,
            lng,
            is_default,
            created_at: now,
            updated_at: now,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        name: String,
        phone: String,
        detail: String,
        lat: Option<f64>,
        lng: Option<f64>,
        is_default: bool,
    ) -> Result<(), DomainError> {
        Self::validate_name(&name)?;
        Self::validate_phone(&phone)?;
        Self::validate_detail(&detail)?;
        Self::validate_geo(lat, lng)?;

        self.name = name;
        self.phone = phone;
        self.detail = detail;
        self.lat = lat;
        self.lng = lng;
        self.is_default = is_default;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn set_default(&mut self, is_default: bool) {
        self.is_default = is_default;
        self.updated_at = Utc::now();
    }

    fn validate_name(name: &str) -> Result<(), DomainError> {
        if name.trim().is_empty() {
            return Err(DomainError::Validation("address name is required".into()));
        }
        Ok(())
    }

    fn validate_phone(phone: &str) -> Result<(), DomainError> {
        let phone = phone.trim();
        let valid = phone.len() == 11
            && phone.starts_with('1')
            && phone.chars().all(|char| char.is_ascii_digit());
        if !valid {
            return Err(DomainError::Validation("invalid phone".into()));
        }
        Ok(())
    }

    fn validate_detail(detail: &str) -> Result<(), DomainError> {
        if detail.trim().is_empty() {
            return Err(DomainError::Validation("address detail is required".into()));
        }
        Ok(())
    }

    fn validate_geo(lat: Option<f64>, lng: Option<f64>) -> Result<(), DomainError> {
        if lat.is_some() ^ lng.is_some() {
            return Err(DomainError::Validation(
                "lat and lng must be both set or both empty".into(),
            ));
        }
        Ok(())
    }
}
