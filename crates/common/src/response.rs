//! 统一响应格式模块

use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use utoipa::ToSchema;

/// 统一 API 响应格式
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetail>,
}

/// 错误详情
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<FieldError>>,
}

/// 字段错误详情
#[derive(Debug, Serialize, ToSchema)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

impl<T: Serialize> ApiResponse<T> {
    /// 创建成功响应
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// 创建错误响应
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ErrorDetail {
                code: code.into(),
                message: message.into(),
                details: None,
            }),
        }
    }

    /// 创建带详情的错误响应
    pub fn error_with_details(
        code: impl Into<String>,
        message: impl Into<String>,
        details: Vec<FieldError>,
    ) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ErrorDetail {
                code: code.into(),
                message: message.into(),
                details: Some(details),
            }),
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

/// 分页响应
#[derive(Debug, Serialize, ToSchema)]
pub struct PagedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

impl<T: Serialize> PagedResponse<T> {
    pub fn new(items: Vec<T>, total: i64, page: i64, page_size: i64) -> Self {
        let total_pages = (total as f64 / page_size as f64).ceil() as i64;
        Self {
            items,
            total,
            page,
            page_size,
            total_pages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success("hello");
        assert!(response.success);
        assert_eq!(response.data, Some("hello"));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_paged_response() {
        let response = PagedResponse::new(vec![1, 2, 3], 10, 1, 3);
        assert_eq!(response.total_pages, 4);
    }
}
