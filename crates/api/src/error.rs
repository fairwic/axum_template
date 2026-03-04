use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_core_kernel::AppError;
use serde_json::json;

pub struct ApiError(pub AppError);

pub type ApiResult<T> = Result<T, ApiError>;

fn map_error_status(error: &AppError) -> StatusCode {
    match error {
        AppError::Validation(_) => StatusCode::BAD_REQUEST,
        AppError::NotFound(_) => StatusCode::NOT_FOUND,
        AppError::Conflict(_) => StatusCode::CONFLICT,
        AppError::Unauthorized => StatusCode::UNAUTHORIZED,
        AppError::Forbidden => StatusCode::FORBIDDEN,
        AppError::Domain(domain) => match domain {
            axum_core_kernel::DomainError::Validation(_) => StatusCode::BAD_REQUEST,
            axum_core_kernel::DomainError::BusinessRule(_) => StatusCode::UNPROCESSABLE_ENTITY,
            axum_core_kernel::DomainError::NotFound(_) => StatusCode::NOT_FOUND,
            axum_core_kernel::DomainError::State(_)
            | axum_core_kernel::DomainError::InvalidState(_)
            | axum_core_kernel::DomainError::ConcurrencyConflict => StatusCode::CONFLICT,
            axum_core_kernel::DomainError::PermissionDenied(_) => StatusCode::FORBIDDEN,
            axum_core_kernel::DomainError::InfrastructureError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            axum_core_kernel::DomainError::AdapterNotFound(_) => StatusCode::NOT_FOUND,
            axum_core_kernel::DomainError::InvalidPayload(_) => StatusCode::BAD_REQUEST,
            axum_core_kernel::DomainError::Storage(_)
            | axum_core_kernel::DomainError::EventPublish(_) => StatusCode::INTERNAL_SERVER_ERROR,
        },
        AppError::Database(_) | AppError::Internal(_) | AppError::Serialization(_) => {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

impl From<AppError> for ApiError {
    fn from(value: AppError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let error = self.0;
        tracing::error!(error = %error, "请求处理失败");

        let status = map_error_status(&error);
        let (code, message) = match &error {
            AppError::Validation(msg) => ("VALIDATION_ERROR", msg.as_str()),
            AppError::NotFound(msg) => ("NOT_FOUND", msg.as_str()),
            AppError::Conflict(msg) => ("CONFLICT", msg.as_str()),
            AppError::Unauthorized => ("UNAUTHORIZED", "未授权访问"),
            AppError::Forbidden => ("FORBIDDEN", "禁止访问"),
            AppError::Domain(e) => match e {
                axum_core_kernel::DomainError::Validation(msg) => {
                    ("DOMAIN_VALIDATION", msg.as_str())
                }
                axum_core_kernel::DomainError::BusinessRule(msg) => ("BUSINESS_RULE", msg.as_str()),
                axum_core_kernel::DomainError::NotFound(msg) => ("NOT_FOUND", msg.as_str()),
                axum_core_kernel::DomainError::State(msg) => ("INVALID_STATE", msg.as_str()),
                axum_core_kernel::DomainError::InvalidState(msg) => ("INVALID_STATE", msg.as_str()),
                axum_core_kernel::DomainError::PermissionDenied(msg) => {
                    ("PERMISSION_DENIED", msg.as_str())
                }
                axum_core_kernel::DomainError::ConcurrencyConflict => {
                    ("CONCURRENCY_CONFLICT", "数据已被修改，请刷新后重试")
                }
                axum_core_kernel::DomainError::InfrastructureError(msg) => {
                    ("INFRASTRUCTURE_ERROR", msg.as_str())
                }
                axum_core_kernel::DomainError::AdapterNotFound(msg) => {
                    ("ADAPTER_NOT_FOUND", msg.as_str())
                }
                axum_core_kernel::DomainError::InvalidPayload(msg) => {
                    ("INVALID_PAYLOAD", msg.as_str())
                }
                axum_core_kernel::DomainError::Storage(msg) => ("STORAGE_ERROR", msg.as_str()),
                axum_core_kernel::DomainError::EventPublish(msg) => {
                    ("EVENT_PUBLISH_ERROR", msg.as_str())
                }
            },
            AppError::Database(_) | AppError::Internal(_) | AppError::Serialization(_) => {
                ("INTERNAL_ERROR", "服务器内部错误")
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maps_validation_and_not_found_to_restful_status() {
        assert_eq!(
            map_error_status(&AppError::Validation("x".into())),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            map_error_status(&AppError::NotFound("x".into())),
            StatusCode::NOT_FOUND
        );
    }

    #[test]
    fn test_maps_domain_invalid_state_to_conflict() {
        let error = AppError::Domain(axum_core_kernel::DomainError::InvalidState("x".into()));
        assert_eq!(map_error_status(&error), StatusCode::CONFLICT);
    }
}
