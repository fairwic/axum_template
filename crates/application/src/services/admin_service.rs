//! Admin service

use std::sync::Arc;

use axum_common::{AppError, AppResult};
use axum_domain::admin::entity::{Admin, AdminRole};
use axum_domain::admin::repo::AdminRepository;
use bcrypt::{hash, verify, DEFAULT_COST};
use ulid::Ulid;

#[derive(Clone)]
pub struct AdminService {
    repo: Arc<dyn AdminRepository>,
}

impl AdminService {
    pub fn new(repo: Arc<dyn AdminRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_admin(
        &self,
        phone: String,
        password: String,
        role: AdminRole,
        store_id: Option<Ulid>,
    ) -> AppResult<Admin> {
        let password_hash =
            hash(password, DEFAULT_COST).map_err(|err| AppError::Internal(err.to_string()))?;
        let admin = Admin::new(phone, password_hash, role, store_id)?;
        self.repo.create(&admin).await
    }

    pub async fn login(&self, phone: &str, password: &str) -> AppResult<Admin> {
        let admin = self
            .repo
            .find_by_phone(phone)
            .await?
            .ok_or_else(|| AppError::NotFound("admin not found".into()))?;
        if !verify(password, &admin.password_hash).unwrap_or(false) {
            return Err(AppError::Validation("password incorrect".into()));
        }
        Ok(admin)
    }
}
