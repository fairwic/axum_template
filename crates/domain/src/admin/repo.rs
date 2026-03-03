//! Admin repository trait

use crate::admin::entity::Admin;
use async_trait::async_trait;
use axum_common::AppResult;

#[async_trait]
pub trait AdminRepository: Send + Sync {
    async fn find_by_phone(&self, phone: &str) -> AppResult<Option<Admin>>;
    async fn create(&self, admin: &Admin) -> AppResult<Admin>;
}
