use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_common::AppError;
use serde_json::json;
use std::sync::OnceLock;

pub struct ApiError(pub AppError);

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ApiErrorMode {
    #[default]
    LegacyOk,
    Restful,
}

static API_ERROR_MODE: OnceLock<ApiErrorMode> = OnceLock::new();

pub fn configure_api_error_mode(mode: ApiErrorMode) {
    if let Some(existing) = API_ERROR_MODE.get() {
        if *existing != mode {
            tracing::warn!(
                existing = ?existing,
                incoming = ?mode,
                "api error mode already configured, ignoring new value"
            );
        }
        return;
    }
    let _ = API_ERROR_MODE.set(mode);
}

fn current_mode() -> ApiErrorMode {
    *API_ERROR_MODE.get_or_init(|| ApiErrorMode::LegacyOk)
}

fn map_error_status(error: &AppError, mode: ApiErrorMode) -> StatusCode {
    match mode {
        ApiErrorMode::LegacyOk => match error {
            AppError::Validation(_) => StatusCode::OK,
            AppError::NotFound(_) => StatusCode::OK,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::Domain(_) => StatusCode::OK,
            AppError::Database(_) | AppError::Internal(_) | AppError::Serialization(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        },
        ApiErrorMode::Restful => match error {
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::Domain(domain) => match domain {
                axum_common::DomainError::Validation(_) => StatusCode::BAD_REQUEST,
                axum_common::DomainError::BusinessRule(_) => StatusCode::UNPROCESSABLE_ENTITY,
                axum_common::DomainError::NotFound(_) => StatusCode::NOT_FOUND,
                axum_common::DomainError::State(_)
                | axum_common::DomainError::InvalidState(_)
                | axum_common::DomainError::ConcurrencyConflict => StatusCode::CONFLICT,
                axum_common::DomainError::PermissionDenied(_) => StatusCode::FORBIDDEN,
                axum_common::DomainError::InfrastructureError(_) => {
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            },
            AppError::Database(_) | AppError::Internal(_) | AppError::Serialization(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        },
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
        let mode = current_mode();
        tracing::error!(error = %error, "请求处理失败");

        let status = map_error_status(&error, mode);
        let (code, message) = match &error {
            AppError::Validation(msg) => ("VALIDATION_ERROR", msg.as_str()),
            AppError::NotFound(msg) => {
                tracing::info!(mode = ?mode, "Converting NotFound error to response");
                ("NOT_FOUND", msg.as_str())
            }
            AppError::Conflict(msg) => ("CONFLICT", msg.as_str()),
            AppError::Unauthorized => ("UNAUTHORIZED", "未授权访问"),
            AppError::Forbidden => ("FORBIDDEN", "禁止访问"),
            AppError::Domain(e) => match e {
                axum_common::DomainError::Validation(msg) => ("DOMAIN_VALIDATION", msg.as_str()),
                axum_common::DomainError::BusinessRule(msg) => ("BUSINESS_RULE", msg.as_str()),
                axum_common::DomainError::NotFound(msg) => ("NOT_FOUND", msg.as_str()),
                axum_common::DomainError::State(msg) => ("INVALID_STATE", msg.as_str()),
                axum_common::DomainError::InvalidState(msg) => ("INVALID_STATE", msg.as_str()),
                axum_common::DomainError::PermissionDenied(msg) => {
                    ("PERMISSION_DENIED", msg.as_str())
                }
                axum_common::DomainError::ConcurrencyConflict => {
                    ("CONCURRENCY_CONFLICT", "数据已被修改，请刷新后重试")
                }
                axum_common::DomainError::InfrastructureError(msg) => {
                    ("INFRASTRUCTURE_ERROR", msg.as_str())
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
    fn test_legacy_mode_keeps_validation_and_not_found_as_ok() {
        assert_eq!(
            map_error_status(&AppError::Validation("x".into()), ApiErrorMode::LegacyOk),
            StatusCode::OK
        );
        assert_eq!(
            map_error_status(&AppError::NotFound("x".into()), ApiErrorMode::LegacyOk),
            StatusCode::OK
        );
    }

    #[test]
    fn test_restful_mode_maps_validation_and_not_found() {
        assert_eq!(
            map_error_status(&AppError::Validation("x".into()), ApiErrorMode::Restful),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            map_error_status(&AppError::NotFound("x".into()), ApiErrorMode::Restful),
            StatusCode::NOT_FOUND
        );
    }

    #[test]
    fn test_restful_mode_maps_domain_invalid_state_to_conflict() {
        let error = AppError::Domain(axum_common::DomainError::InvalidState("x".into()));
        assert_eq!(
            map_error_status(&error, ApiErrorMode::Restful),
            StatusCode::CONFLICT
        );
    }
}
