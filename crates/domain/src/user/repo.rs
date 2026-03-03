//! User repository trait

use crate::user::entity::User;
use axum_common::AppResult;
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_openid(&self, openid: &str) -> AppResult<Option<User>>;
    async fn create(&self, user: &User) -> AppResult<User>;
}
