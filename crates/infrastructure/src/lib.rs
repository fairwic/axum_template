//! # Backend Template Infrastructure
//!
//! 基础设施层，仅保留最小 Postgres 用户仓储。

pub mod config;

pub mod models {
    pub mod user_model;
}

pub mod postgres {
    pub mod user_repo;
}

pub use config::AppConfig;
pub use postgres::user_repo::PgUserRepository;
