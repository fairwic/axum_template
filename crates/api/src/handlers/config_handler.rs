//! Config handlers

use axum::{extract::State, Json};
use axum_common::{ApiResponse, AppError, AppResult};

use crate::dtos::config_dto::{ConfigResponse, UpdateConfigRequest};
use crate::state::{AppState, BizConfig};

fn to_response(config: &BizConfig) -> ConfigResponse {
    ConfigResponse {
        delivery_free_radius_km: config.delivery_free_radius_km,
        runner_service_fee: config.runner_service_fee,
        customer_service_phone: config.customer_service_phone.clone(),
        runner_banner_enabled: config.runner_banner_enabled,
        runner_banner_text: config.runner_banner_text.clone(),
        pay_timeout_secs: config.pay_timeout_secs,
        auto_accept_secs: config.auto_accept_secs,
        cancel_timeout_secs: config.cancel_timeout_secs,
    }
}

fn validate_update(payload: &UpdateConfigRequest) -> AppResult<()> {
    if payload.delivery_free_radius_km < 0.0 {
        return Err(AppError::Validation(
            "delivery_free_radius_km must be >= 0".into(),
        ));
    }
    if payload.runner_service_fee < 0 {
        return Err(AppError::Validation(
            "runner_service_fee must be >= 0".into(),
        ));
    }
    if payload.customer_service_phone.trim().is_empty() {
        return Err(AppError::Validation(
            "customer_service_phone is required".into(),
        ));
    }
    if payload.runner_banner_text.trim().is_empty() {
        return Err(AppError::Validation(
            "runner_banner_text is required".into(),
        ));
    }
    if payload.pay_timeout_secs == 0 {
        return Err(AppError::Validation("pay_timeout_secs must be > 0".into()));
    }
    if payload.auto_accept_secs == 0 {
        return Err(AppError::Validation("auto_accept_secs must be > 0".into()));
    }
    if payload.cancel_timeout_secs == 0 {
        return Err(AppError::Validation(
            "cancel_timeout_secs must be > 0".into(),
        ));
    }
    Ok(())
}

#[utoipa::path(
    get,
    path = "/config",
    responses((status = 200, body = ApiResponse<ConfigResponse>)),
    tag = "Config"
)]
/// 接口功能：get_config，获取小程序全局配置
pub async fn get_config(State(state): State<AppState>) -> AppResult<ApiResponse<ConfigResponse>> {
    let config = state.biz_config.read().await;
    let config = to_response(&config);
    Ok(ApiResponse::success(config))
}

#[utoipa::path(
    get,
    path = "/admin/config",
    responses((status = 200, body = ApiResponse<ConfigResponse>)),
    tag = "AdminConfig"
)]
/// 接口功能：admin_get_config，后台查询全局配置
pub async fn admin_get_config(
    State(state): State<AppState>,
) -> AppResult<ApiResponse<ConfigResponse>> {
    let config = state.biz_config.read().await;
    let config = to_response(&config);
    Ok(ApiResponse::success(config))
}

#[utoipa::path(
    put,
    path = "/admin/config",
    request_body = UpdateConfigRequest,
    responses((status = 200, body = ApiResponse<ConfigResponse>)),
    tag = "AdminConfig"
)]
/// 接口功能：admin_update_config，后台更新全局配置
pub async fn admin_update_config(
    State(state): State<AppState>,
    Json(payload): Json<UpdateConfigRequest>,
) -> AppResult<ApiResponse<ConfigResponse>> {
    validate_update(&payload)?;
    let updated = {
        let mut config = state.biz_config.write().await;
        config.delivery_free_radius_km = payload.delivery_free_radius_km;
        config.runner_service_fee = payload.runner_service_fee;
        config.customer_service_phone = payload.customer_service_phone;
        config.runner_banner_enabled = payload.runner_banner_enabled;
        config.runner_banner_text = payload.runner_banner_text;
        config.pay_timeout_secs = payload.pay_timeout_secs;
        config.auto_accept_secs = payload.auto_accept_secs;
        config.cancel_timeout_secs = payload.cancel_timeout_secs;
        config.clone()
    };

    Ok(ApiResponse::success(to_response(&updated)))
}
