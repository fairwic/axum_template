use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_common::AppError;
use serde_json::json;

pub struct ApiError(pub AppError);

pub type ApiResult<T> = Result<T, ApiError>;

impl From<AppError> for ApiError {
    fn from(value: AppError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let error = self.0;
        tracing::error!(error = %error, "请求处理失败");

        let (status, code, message) = match &error {
            AppError::Validation(msg) => (StatusCode::OK, "VALIDATION_ERROR", msg.as_str()),
            AppError::NotFound(msg) => {
                tracing::info!("Converting NotFound error to OK 200: {}", msg);
                (StatusCode::OK, "NOT_FOUND", msg.as_str())
            }
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.as_str()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "未授权访问"),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN", "禁止访问"),
            AppError::Domain(e) => match e {
                axum_common::DomainError::Validation(msg) => {
                    (StatusCode::OK, "DOMAIN_VALIDATION", msg.as_str())
                }
                axum_common::DomainError::BusinessRule(msg) => {
                    (StatusCode::OK, "BUSINESS_RULE", msg.as_str())
                }
                axum_common::DomainError::NotFound(msg) => {
                    (StatusCode::OK, "NOT_FOUND", msg.as_str())
                }
                axum_common::DomainError::State(msg) => {
                    (StatusCode::OK, "INVALID_STATE", msg.as_str())
                }
                axum_common::DomainError::InvalidState(msg) => {
                    (StatusCode::OK, "INVALID_STATE", msg.as_str())
                }
                axum_common::DomainError::PermissionDenied(msg) => {
                    (StatusCode::OK, "PERMISSION_DENIED", msg.as_str())
                }
                axum_common::DomainError::ConcurrencyConflict => (
                    StatusCode::OK,
                    "CONCURRENCY_CONFLICT",
                    "数据已被修改，请刷新后重试",
                ),
                axum_common::DomainError::InfrastructureError(msg) => {
                    (StatusCode::OK, "INFRASTRUCTURE_ERROR", msg.as_str())
                }
            },
            AppError::Database(_) | AppError::Internal(_) | AppError::Serialization(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "服务器内部错误",
            ),
        };

        let body = Json(json!({
            "success": false,
            "error": {
                "code": code,
                "message": message
            }
        }));

        (status, body).into_response()
    }
}
