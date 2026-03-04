//! Health check handler

use axum_common_api::ApiResponse;

/// 接口功能：health_check，服务健康检查
pub async fn health_check() -> ApiResponse<&'static str> {
    ApiResponse::success("ok")
}
