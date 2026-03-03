//! Category entity

use crate::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CategoryStatus {
    On,
    Off,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Category {
    pub id: Ulid,
    pub store_id: Ulid,
    pub name: String,
    pub sort_order: i32,
    pub status: CategoryStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Category {
    pub fn new(
        store_id: Ulid,
        name: String,
        sort_order: i32,
        status: CategoryStatus,
    ) -> Result<Self, DomainError> {
        if name.trim().is_empty() {
            return Err(DomainError::Validation("category name is required".into()));
        }
        let now = Utc::now();
        Ok(Self {
            id: Ulid::new(),
            store_id,
            name,
            sort_order,
            status,
            created_at: now,
            updated_at: now,
        })
    }
}
