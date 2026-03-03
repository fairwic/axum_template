//! Member handlers

use axum_common::{ApiResponse, AppResult};
use utoipa::ToSchema;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct MemberStatusResponse {
    pub is_member: bool,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct MemberBenefitsResponse {
    pub benefits: Vec<String>,
}

#[utoipa::path(
    get,
    path = "/member/status",
    responses((status = 200, description = "Member status", body = ApiResponse<MemberStatusResponse>)),
    tag = "Member"
)]
pub async fn member_status(
) -> AppResult<ApiResponse<MemberStatusResponse>> {
    let is_member = true;
    Ok(ApiResponse::success(MemberStatusResponse { is_member }))
}

#[utoipa::path(
    get,
    path = "/member/benefits",
    responses((status = 200, description = "Member benefits", body = ApiResponse<MemberBenefitsResponse>)),
    tag = "Member"
)]
pub async fn member_benefits() -> AppResult<ApiResponse<MemberBenefitsResponse>> {
    let benefits = vec![
        "3km 内免配送费".to_string(),
        "专属券入口".to_string(),
    ];
    Ok(ApiResponse::success(MemberBenefitsResponse { benefits }))
}
