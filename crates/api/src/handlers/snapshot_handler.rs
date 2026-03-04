use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

use axum_api_common::ApiResponse;
use axum_application::snapshot::ingest_service::IngestService;
use axum_domain::snapshot::model::Platform;

#[utoipa::path(
    post,
    path = "/v1/ingest/{platform}/product",
    params(("platform" = String, Path, description = "平台名称(如 temu, yandex)")),
    request_body = Value,
    responses((status = 200, body = ApiResponse<Value>)),
    tag = "Snapshot"
)]
/// 接口功能：ingest_product，摄取产品数据快照
pub async fn ingest_product(
    State(ingest_service): State<Arc<IngestService>>,
    Path(platform_str): Path<String>,
    Json(payload): Json<Value>,
) -> crate::error::ApiResult<ApiResponse<Value>> {
    let platform = Platform::parse(&platform_str);
    // 强制转换为 AppError 等应用错误，因为 ingest_service 返回 DomainError
    let trace_id = ingest_service
        .ingest_product(platform, payload)
        .await
        .map_err(|e| axum_core_kernel::AppError::Internal(e.to_string()))?;

    Ok(ApiResponse::success(json!({"trace_id": trace_id})))
}

#[utoipa::path(
    post,
    path = "/v1/ingest/{platform}/shop",
    params(("platform" = String, Path, description = "平台名称(如 temu, yandex)")),
    request_body = Value,
    responses((status = 200, body = ApiResponse<Value>)),
    tag = "Snapshot"
)]
/// 接口功能：ingest_shop，摄取店铺数据快照
pub async fn ingest_shop(
    State(ingest_service): State<Arc<IngestService>>,
    Path(platform_str): Path<String>,
    Json(payload): Json<Value>,
) -> crate::error::ApiResult<ApiResponse<Value>> {
    let platform = Platform::parse(&platform_str);
    let trace_id = ingest_service
        .ingest_shop(platform, payload)
        .await
        .map_err(|e| axum_core_kernel::AppError::Internal(e.to_string()))?;

    Ok(ApiResponse::success(json!({"trace_id": trace_id})))
}
