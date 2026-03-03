//! Auth handlers

use axum::{extract::State, Json};
use axum_application::dtos::user_dto::{
    PhoneSmsLoginDto, SendSmsCodeDto, UserResponse, WechatLoginDto,
};
use axum_common::{ApiResponse, AppError, AppResult};
use chrono::Utc;
use utoipa::ToSchema;

use crate::auth::jwt::{encode_token, Claims};
use crate::state::AppState;

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, serde::Serialize, ToSchema)]
pub struct SendSmsCodeResponse {
    pub expires_in_secs: u64,
}

#[utoipa::path(
    post,
    path = "/auth/wechat_login",
    request_body = WechatLoginDto,
    responses((status = 200, description = "Login success", body = ApiResponse<LoginResponse>)),
    tag = "Auth"
)]
pub async fn wechat_login(
    State(state): State<AppState>,
    Json(payload): Json<WechatLoginDto>,
) -> AppResult<ApiResponse<LoginResponse>> {
    let user = state
        .user_service
        .login_with_wechat_code(payload.code, payload.nickname, payload.avatar)
        .await?;

    Ok(ApiResponse::success(build_login_response(&state, user)?))
}

#[utoipa::path(
    post,
    path = "/auth/sms/send_code",
    request_body = SendSmsCodeDto,
    responses((status = 200, description = "SMS code sent", body = ApiResponse<SendSmsCodeResponse>)),
    tag = "Auth"
)]
pub async fn send_sms_code(
    State(state): State<AppState>,
    Json(payload): Json<SendSmsCodeDto>,
) -> AppResult<ApiResponse<SendSmsCodeResponse>> {
    state
        .user_service
        .send_login_sms_code(payload.phone)
        .await?;

    Ok(ApiResponse::success(SendSmsCodeResponse {
        expires_in_secs: state.sms_code_ttl_secs,
    }))
}

#[utoipa::path(
    post,
    path = "/auth/phone_sms_login",
    request_body = PhoneSmsLoginDto,
    responses((status = 200, description = "Phone SMS login success", body = ApiResponse<LoginResponse>)),
    tag = "Auth"
)]
pub async fn phone_sms_login(
    State(state): State<AppState>,
    Json(payload): Json<PhoneSmsLoginDto>,
) -> AppResult<ApiResponse<LoginResponse>> {
    let user = state
        .user_service
        .login_with_phone_sms(
            payload.phone,
            payload.sms_code,
            payload.wechat_code,
            payload.nickname,
            payload.avatar,
        )
        .await?;

    Ok(ApiResponse::success(build_login_response(&state, user)?))
}

fn build_login_response(state: &AppState, user: axum_domain::User) -> AppResult<LoginResponse> {
    let exp = (Utc::now().timestamp() as u64 + state.jwt_ttl_secs) as usize;
    let claims = Claims {
        sub: user.id.to_string(),
        role: "USER".into(),
        exp,
    };

    let token = encode_token(&claims, &state.jwt_secret)
        .map_err(|err| AppError::Internal(err.to_string()))?;
    let response = LoginResponse {
        token,
        user: UserResponse::from(user),
    };

    Ok(response)
}
