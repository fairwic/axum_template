//! Health check handler

use axum_common::ApiResponse;

pub async fn health_check() -> ApiResponse<&'static str> {
    ApiResponse::success("ok")
}
