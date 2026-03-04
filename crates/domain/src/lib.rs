//! # Backend Template Domain
//!
//! 领域层，仅保留最小 User 聚合示例。

pub mod address;
pub mod admin;
pub mod auth;
pub mod cache;
pub mod cart;
pub mod category;
pub mod error;
pub mod order;
pub mod product;
pub mod runner_order;
pub mod store;
pub mod transaction;
pub mod user;

pub type JsonValue = serde_json::Value;

pub use address::entity::Address;
pub use address::repo::AddressRepository;
pub use admin::entity::{Admin, AdminRole};
pub use admin::repo::AdminRepository;
pub use auth::{SmsGateway, WechatAuthClient, WechatSession};
pub use cache::CacheService;
pub use cart::entity::{Cart, CartItem};
pub use cart::repo::CartRepository;
pub use category::entity::{Category, CategoryStatus};
pub use category::repo::CategoryRepository;
pub use error::DomainError;
pub use order::entity::{DeliveryType, GoodsOrder, GoodsOrderItem, GoodsOrderStatus, PayStatus};
pub use order::repo::GoodsOrderRepository;
pub use product::entity::{Product, ProductStatus};
pub use product::repo::ProductRepository;
pub use runner_order::entity::{RunnerOrder, RunnerOrderStatus};
pub use runner_order::repo::RunnerOrderRepository;
pub use store::entity::{Store, StoreStatus};
pub use store::repo::StoreRepository;
pub use transaction::{OrderUnitOfWork, TransactionManager};
pub use user::entity::User;
pub use user::repo::UserRepository;
