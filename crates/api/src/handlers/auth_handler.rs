//! Auth handlers

use axum::{extract::State, Json};
use axum_application::dtos::user_dto::{UserResponse, WechatLoginDto};
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
        .login_with_openid(payload.openid, payload.nickname, payload.avatar)
        .await?;

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

    Ok(ApiResponse::success(response))
}
