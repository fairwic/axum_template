//! # Backend Template Infrastructure
//!
//! 基础设施层，仅保留最小 Postgres 用户仓储。

pub mod config;

pub mod models {
    pub mod address_model;
}

pub mod postgres {
    pub mod address_repo;
}

pub mod redis {
    pub mod cache;
}

pub mod memory {
    pub mod cache;
}

pub mod external {
    pub mod sms_gateway;
    pub mod wechat_auth_client;
}

pub use config::{AppConfig, CacheProvider, LbsProvider, RuntimeConfig};
pub use external::sms_gateway::LogSmsGateway;
pub use external::wechat_auth_client::WechatMiniProgramClient;
pub use memory::cache::MemoryCacheService;
pub use postgres::address_repo::PgAddressRepository;
pub use redis::cache::RedisCacheService;
