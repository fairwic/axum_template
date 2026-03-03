//! User repository trait

use crate::user::entity::User;
use async_trait::async_trait;
use axum_core_kernel::AppResult;
use ulid::Ulid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_openid(&self, openid: &str) -> AppResult<Option<User>>;
    async fn find_by_id(&self, user_id: Ulid) -> AppResult<Option<User>>;
    async fn find_by_phone(&self, phone: &str) -> AppResult<Option<User>>;
    async fn create(&self, user: &User) -> AppResult<User>;
    async fn set_current_store(&self, user_id: Ulid, store_id: Ulid) -> AppResult<User>;
    async fn bind_phone(&self, user_id: Ulid, phone: String) -> AppResult<User>;
}
