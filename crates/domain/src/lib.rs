//! # Backend Template Domain
//!
//! 领域层，仅保留最小 User 聚合示例。

pub mod error;
pub mod cache;
pub mod user;

pub use error::DomainError;
pub use cache::CacheService;
pub use user::entity::User;
pub use user::repo::UserRepository;
