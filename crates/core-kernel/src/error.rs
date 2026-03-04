//! Unified error model used across layers.

use thiserror::Error;

/// Domain-level business errors.
#[derive(Error, Debug)]
pub enum DomainError {
    #[error("验证失败: {0}")]
    Validation(String),

    #[error("业务规则违反: {0}")]
    BusinessRule(String),

    #[error("资源未找到: {0}")]
    NotFound(String),

    #[error("状态错误: {0}")]
    InvalidState(String),

    #[error("权限不足: {0}")]
    PermissionDenied(String),

    #[error("状态错误: {0}")]
    State(String),

    #[error("数据已被修改，请刷新后重试")]
    ConcurrencyConflict,

    #[error("基础设施错误: {0}")]
    InfrastructureError(String),

    #[error("找不到对应平台适配器: {0}")]
    AdapterNotFound(String),

    #[error("无效的数据载荷: {0}")]
    InvalidPayload(String),

    #[error("存储错误: {0}")]
    Storage(String),

    #[error("事件发布错误: {0}")]
    EventPublish(String),
}

/// Application-level unified error.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    Validation(String),

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Conflict(String),

    #[error("未授权访问")]
    Unauthorized,

    #[error("禁止访问")]
    Forbidden,

    #[error("数据库错误: {0}")]
    Database(String),

    #[error("领域错误: {0}")]
    Domain(#[from] DomainError),

    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("内部错误: {0}")]
    Internal(String),
}

impl AppError {
    pub fn database<E: std::fmt::Display>(error: E) -> Self {
        Self::Database(error.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
