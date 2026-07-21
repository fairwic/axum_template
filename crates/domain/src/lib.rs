//! # Backend Template Domain
//!
//! 领域层，仅保留最小 User 聚合示例。

pub mod address;
pub mod cache;
pub mod error;
pub mod snapshot;

pub type JsonValue = serde_json::Value;

pub use address::entity::Address;
pub use address::repo::AddressRepository;
pub use cache::CacheService;
pub use error::DomainError;
