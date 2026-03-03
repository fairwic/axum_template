//! # Backend Template Infrastructure
//!
//! 基础设施层，仅保留最小 Postgres 用户仓储。

pub mod config;

pub mod models {
    pub mod admin_model;
    pub mod store_model;
    pub mod user_model;
}

pub mod postgres {
    pub mod admin_repo;
    pub mod store_repo;
    pub mod user_repo;
}

pub mod redis {
    pub mod cache;
}

pub use config::AppConfig;
pub use postgres::admin_repo::PgAdminRepository;
pub use postgres::store_repo::PgStoreRepository;
pub use postgres::user_repo::PgUserRepository;
pub use redis::cache::RedisCacheService;
