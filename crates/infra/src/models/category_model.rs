//! Category model for persistence

use axum_core_kernel::AppError;
use axum_domain::category::entity::{Category, CategoryStatus};
use chrono::{DateTime, Utc};
use ulid::Ulid;

#[derive(Debug, sqlx::FromRow)]
pub struct CategoryModel {
    pub id: String,
    pub store_id: String,
    pub name: String,
    pub sort_order: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CategoryModel {
    pub fn from_entity(category: &Category) -> Self {
        Self {
            id: category.id.to_string(),
            store_id: category.store_id.to_string(),
            name: category.name.clone(),
            sort_order: category.sort_order,
            status: match category.status {
                CategoryStatus::On => "ON".to_string(),
                CategoryStatus::Off => "OFF".to_string(),
            },
            created_at: category.created_at,
            updated_at: category.updated_at,
        }
    }

    pub fn into_entity(self) -> Result<Category, AppError> {
        let id = Ulid::from_string(&self.id)
            .map_err(|_| AppError::Internal("invalid ulid in database".into()))?;
        let store_id = Ulid::from_string(&self.store_id)
            .map_err(|_| AppError::Internal("invalid store_id in database".into()))?;
        let status = match self.status.as_str() {
            "ON" => CategoryStatus::On,
            "OFF" => CategoryStatus::Off,
            _ => return Err(AppError::Internal("invalid category status".into())),
        };
        Ok(Category {
            id,
            store_id,
            name: self.name,
            sort_order: self.sort_order,
            status,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
