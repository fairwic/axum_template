//! # BJD Common
//!
//! 通用工具库，包含错误处理、响应格式、常量定义等。

pub mod constants;
pub mod error;
pub mod response;

pub use error::{AppError, AppResult, DomainError};
pub use response::{ApiResponse, PagedResponse};
