//! Address repository trait

use async_trait::async_trait;
use axum_core_kernel::AppResult;
use ulid::Ulid;

use crate::address::entity::Address;

#[async_trait]
pub trait AddressRepository: Send + Sync {
    async fn list_by_user(&self, user_id: Ulid) -> AppResult<Vec<Address>>;
    async fn find_by_id(&self, user_id: Ulid, address_id: Ulid) -> AppResult<Option<Address>>;
    async fn create(&self, address: &Address) -> AppResult<Address>;
    async fn update(&self, address: &Address) -> AppResult<Address>;
    async fn delete(&self, user_id: Ulid, address_id: Ulid) -> AppResult<()>;
}
