//! Auth handlers

use axum::{extract::State, Json};
use axum_common::{ApiResponse, AppError, AppResult};
use chrono::Utc;

use crate::auth::jwt::{encode_token, Claims};
use crate::dtos::auth_dto::{
    LoginResponse, PhoneSmsLoginDto, SendSmsCodeDto, SendSmsCodeResponse, UserResponse,
    WechatLoginDto,
};
use crate::state::AppState;

#[utoipa::path(
    post,
    path = "/auth/wechat_login",
    request_body = WechatLoginDto,
    responses((status = 200, description = "Login success", body = ApiResponse<LoginResponse>)),
    tag = "Auth"
)]
/// 接口功能：wechat_login，微信授权登录并签发令牌
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
/// 接口功能：send_sms_code，发送短信验证码
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
/// 接口功能：phone_sms_login，手机号验证码登录并绑定微信信息
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
