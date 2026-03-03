//! Config handlers

use axum::extract::State;
use axum_common::{ApiResponse, AppResult};

use crate::dtos::config_dto::ConfigResponse;
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/config",
    responses((status = 200, body = ApiResponse<ConfigResponse>)),
    tag = "Config"
)]
/// 接口功能：get_config，获取小程序全局配置
pub async fn get_config(State(state): State<AppState>) -> AppResult<ApiResponse<ConfigResponse>> {
    let config = ConfigResponse {
        delivery_free_radius_km: state.biz_config.delivery_free_radius_km,
        runner_service_fee: state.biz_config.runner_service_fee,
        customer_service_phone: state.biz_config.customer_service_phone.clone(),
        runner_banner_enabled: state.biz_config.runner_banner_enabled,
        runner_banner_text: state.biz_config.runner_banner_text.clone(),
        pay_timeout_secs: state.biz_config.pay_timeout_secs,
        auto_accept_secs: state.biz_config.auto_accept_secs,
    };
    Ok(ApiResponse::success(config))
}
