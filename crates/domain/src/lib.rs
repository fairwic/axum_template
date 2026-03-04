//! # Backend Template Domain
//!
//! 领域层，仅保留最小 User 聚合示例。

pub mod address;
pub mod cache;
pub mod error;
pub mod snapshot;
pub mod store;
pub mod user;

pub type JsonValue = serde_json::Value;

pub use address::entity::Address;
pub use address::repo::AddressRepository;
pub use cache::CacheService;
pub use error::DomainError;
pub use store::entity::{Store, StoreStatus};
pub use store::repo::StoreRepository;
pub use user::entity::User;
pub use user::repo::UserRepository;
