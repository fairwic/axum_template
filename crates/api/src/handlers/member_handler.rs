//! Member handlers

use axum_api_common::ApiResponse;

use crate::dtos::member_dto::{MemberBenefitsResponse, MemberStatusResponse};

#[utoipa::path(
    get,
    path = "/member/status",
    responses((status = 200, description = "Member status", body = ApiResponse<MemberStatusResponse>)),
    tag = "Member"
)]
/// 接口功能：member_status，获取当前会员状态
pub async fn member_status() -> crate::error::ApiResult<ApiResponse<MemberStatusResponse>> {
    let is_member = true;
    Ok(ApiResponse::success(MemberStatusResponse { is_member }))
}

#[utoipa::path(
    get,
    path = "/member/benefits",
    responses((status = 200, description = "Member benefits", body = ApiResponse<MemberBenefitsResponse>)),
    tag = "Member"
)]
/// 接口功能：member_benefits，获取会员权益说明
pub async fn member_benefits() -> crate::error::ApiResult<ApiResponse<MemberBenefitsResponse>> {
    let benefits = vec!["3km 内免配送费".to_string(), "专属券入口".to_string()];
    Ok(ApiResponse::success(MemberBenefitsResponse { benefits }))
}
