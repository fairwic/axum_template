//! Admin entity

use crate::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AdminRole {
    Platform,
    Store,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Admin {
    pub id: Ulid,
    pub phone: String,
    pub password_hash: String,
    pub role: AdminRole,
    pub store_id: Option<Ulid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Admin {
    pub fn new(
        phone: String,
        password_hash: String,
        role: AdminRole,
        store_id: Option<Ulid>,
    ) -> Result<Self, DomainError> {
        Self::validate_phone(&phone)?;
        Self::validate_role_store(&role, &store_id)?;

        let now = Utc::now();
        Ok(Self {
            id: Ulid::new(),
            phone,
            password_hash,
            role,
            store_id,
            created_at: now,
            updated_at: now,
        })
    }

    fn validate_phone(phone: &str) -> Result<(), DomainError> {
        if phone.trim().is_empty() {
            return Err(DomainError::Validation("phone is required".into()));
        }
        Ok(())
    }

    fn validate_role_store(role: &AdminRole, store_id: &Option<Ulid>) -> Result<(), DomainError> {
        if matches!(role, AdminRole::Store) && store_id.is_none() {
            return Err(DomainError::Validation("store_id is required for store admin".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_requires_phone() {
        let err = Admin::new("".into(), "hash".into(), AdminRole::Platform, None).unwrap_err();
        assert!(err.to_string().contains("phone"));
    }

    #[test]
    fn test_store_admin_requires_store_id() {
        let err = Admin::new("138".into(), "hash".into(), AdminRole::Store, None).unwrap_err();
        assert!(err.to_string().contains("store_id"));
    }
}
