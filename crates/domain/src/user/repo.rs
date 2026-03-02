//! User repository trait

use crate::user::entity::User;
use axum_common::AppResult;
use async_trait::async_trait;
use ulid::Ulid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &User) -> AppResult<User>;
    async fn find_by_id(&self, id: Ulid) -> AppResult<Option<User>>;
    async fn list(&self) -> AppResult<Vec<User>>;
    async fn update(&self, user: &User) -> AppResult<User>;
    async fn delete(&self, id: Ulid) -> AppResult<bool>;
}
