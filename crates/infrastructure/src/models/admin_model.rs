//! Admin model for persistence

use axum_core_kernel::AppError;
use axum_domain::admin::entity::{Admin, AdminRole};
use chrono::{DateTime, Utc};
use ulid::Ulid;

#[derive(Debug, sqlx::FromRow)]
pub struct AdminModel {
    pub id: String,
    pub phone: String,
    pub password_hash: String,
    pub role: String,
    pub store_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AdminModel {
    pub fn from_entity(admin: &Admin) -> Self {
        Self {
            id: admin.id.to_string(),
            phone: admin.phone.clone(),
            password_hash: admin.password_hash.clone(),
            role: match admin.role {
                AdminRole::Platform => "PLATFORM".to_string(),
                AdminRole::Store => "STORE".to_string(),
            },
            store_id: admin.store_id.map(|id| id.to_string()),
            created_at: admin.created_at,
            updated_at: admin.updated_at,
        }
    }

    pub fn into_entity(self) -> Result<Admin, AppError> {
        let id = Ulid::from_string(&self.id)
            .map_err(|_| AppError::Internal("invalid ulid in database".into()))?;
        let role = match self.role.as_str() {
            "PLATFORM" => AdminRole::Platform,
            "STORE" => AdminRole::Store,
            _ => return Err(AppError::Internal("invalid admin role".into())),
        };
        let store_id = match self.store_id {
            Some(value) => Some(
                Ulid::from_string(&value)
                    .map_err(|_| AppError::Internal("invalid store_id in database".into()))?,
            ),
            None => None,
        };

        Ok(Admin {
            id,
            phone: self.phone,
            password_hash: self.password_hash,
            role,
            store_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
