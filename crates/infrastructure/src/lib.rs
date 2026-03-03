//! # Backend Template Infrastructure
//!
//! 基础设施层，仅保留最小 Postgres 用户仓储。

pub mod config;

pub mod models {
    pub mod admin_model;
    pub mod cart_model;
    pub mod category_model;
    pub mod goods_order_model;
    pub mod product_model;
    pub mod runner_order_model;
    pub mod store_model;
    pub mod user_model;
}

pub mod postgres {
    pub mod admin_repo;
    pub mod cart_repo;
    pub mod category_repo;
    pub mod goods_order_repo;
    pub mod product_repo;
    pub mod runner_order_repo;
    pub mod store_repo;
    pub mod user_repo;
}

pub mod redis {
    pub mod cache;
}

pub use config::AppConfig;
pub use postgres::admin_repo::PgAdminRepository;
pub use postgres::cart_repo::PgCartRepository;
pub use postgres::category_repo::PgCategoryRepository;
pub use postgres::goods_order_repo::PgGoodsOrderRepository;
pub use postgres::product_repo::PgProductRepository;
pub use postgres::runner_order_repo::PgRunnerOrderRepository;
pub use postgres::store_repo::PgStoreRepository;
pub use postgres::user_repo::PgUserRepository;
pub use redis::cache::RedisCacheService;
