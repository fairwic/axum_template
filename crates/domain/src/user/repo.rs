//! User repository trait

use crate::user::entity::User;
use async_trait::async_trait;
use axum_common::{AppError, AppResult};
use ulid::Ulid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_openid(&self, openid: &str) -> AppResult<Option<User>>;
    async fn find_by_phone(&self, _phone: &str) -> AppResult<Option<User>> {
        Ok(None)
    }
    async fn create(&self, user: &User) -> AppResult<User>;
    async fn bind_phone(&self, _user_id: Ulid, _phone: String) -> AppResult<User> {
        Err(AppError::Internal("bind_phone is not implemented".into()))
    }
}
