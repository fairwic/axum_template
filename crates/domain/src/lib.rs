//! # Backend Template Domain
//!
//! 领域层，仅保留最小 User 聚合示例。

pub mod error;
pub mod cache;
pub mod admin;
pub mod category;
pub mod product;
pub mod store;
pub mod user;

pub use error::DomainError;
pub use cache::CacheService;
pub use admin::entity::{Admin, AdminRole};
pub use admin::repo::AdminRepository;
pub use category::entity::{Category, CategoryStatus};
pub use category::repo::CategoryRepository;
pub use product::entity::{Product, ProductStatus};
pub use product::repo::ProductRepository;
pub use store::entity::{Store, StoreStatus};
pub use store::repo::StoreRepository;
pub use user::entity::User;
pub use user::repo::UserRepository;
