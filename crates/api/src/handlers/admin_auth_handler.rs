//! Admin auth handlers

use axum::{extract::State, Json};
use axum_common::{ApiResponse, AppError, AppResult};
use chrono::Utc;

use axum_domain::admin::entity::{Admin, AdminRole};

use crate::auth::jwt::{encode_token, Claims};
use crate::dtos::admin_auth_dto::{AdminLoginDto, AdminLoginResponse, AdminResponse};
use crate::state::AppState;

fn role_to_string(role: &AdminRole) -> String {
    match role {
        AdminRole::Platform => "PLATFORM".to_string(),
        AdminRole::Store => "STORE".to_string(),
    }
}

fn to_response(admin: Admin) -> AdminResponse {
    AdminResponse {
        id: admin.id.to_string(),
        phone: admin.phone,
        role: role_to_string(&admin.role),
        store_id: admin.store_id.map(|id| id.to_string()),
    }
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = AdminLoginDto,
    responses((status = 200, description = "Admin login", body = ApiResponse<AdminLoginResponse>)),
    tag = "Admin"
)]
/// 接口功能：admin_login，管理员账号密码登录并签发令牌
pub async fn admin_login(
    State(state): State<AppState>,
    Json(payload): Json<AdminLoginDto>,
) -> AppResult<ApiResponse<AdminLoginResponse>> {
    let admin = state
        .admin_service
        .login(&payload.phone, &payload.password)
        .await?;

    let exp = (Utc::now().timestamp() as u64 + state.jwt_ttl_secs) as usize;
    let claims = Claims {
        sub: admin.id.to_string(),
        role: role_to_string(&admin.role),
        exp,
    };

    let token = encode_token(&claims, &state.jwt_secret)
        .map_err(|err| AppError::Internal(err.to_string()))?;

    let response = AdminLoginResponse {
        token,
        admin: to_response(admin),
    };

    Ok(ApiResponse::success(response))
}
