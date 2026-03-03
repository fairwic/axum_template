//! # Backend Template Application
//!
//! 应用层，仅保留最小 UserService 示例。

pub mod dtos {
    pub mod user_dto;
}

pub mod services {
    pub mod admin_service;
    pub mod user_service;
}

pub use services::admin_service::AdminService;
pub use services::user_service::UserService;
