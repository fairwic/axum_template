//! User handlers

use axum::{extract::{Path, State}, Json};
use axum_application::dtos::user_dto::{CreateUserDto, UpdateUserDto, UserResponse};
use axum_common::{ApiResponse, AppError, AppResult};
use ulid::Ulid;

use crate::state::AppState;

fn parse_ulid(id: &str) -> AppResult<Ulid> {
    Ulid::from_string(id).map_err(|_| AppError::Validation("invalid user id".into()))
}

#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUserDto,
    responses(
        (status = 200, description = "User created", body = ApiResponse<UserResponse>)
    ),
    tag = "User"
)]
/// 接口功能：create_user，创建用户
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserDto>,
) -> AppResult<ApiResponse<UserResponse>> {
    let user = state
        .user_service
        .create_user(payload.name, payload.email)
        .await?;
    Ok(ApiResponse::success(UserResponse::from(user)))
}

#[utoipa::path(
    get,
    path = "/users/{id}",
    params(("id" = String, Path, description = "User id")),
    responses(
        (status = 200, description = "User", body = ApiResponse<UserResponse>)
    ),
    tag = "User"
)]
/// 接口功能：get_user，获取用户详情
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<ApiResponse<UserResponse>> {
    let id = parse_ulid(&id)?;
    let user = state.user_service.get_user(id).await?;
    Ok(ApiResponse::success(UserResponse::from(user)))
}

#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "User list", body = ApiResponse<Vec<UserResponse>>)
    ),
    tag = "User"
)]
/// 接口功能：list_users，查询用户列表
pub async fn list_users(State(state): State<AppState>) -> AppResult<ApiResponse<Vec<UserResponse>>> {
    let users = state.user_service.list_users().await?;
    let data = users.into_iter().map(UserResponse::from).collect();
    Ok(ApiResponse::success(data))
}

#[utoipa::path(
    put,
    path = "/users/{id}",
    request_body = UpdateUserDto,
    params(("id" = String, Path, description = "User id")),
    responses(
        (status = 200, description = "User updated", body = ApiResponse<UserResponse>)
    ),
    tag = "User"
)]
/// 接口功能：update_user，更新用户信息
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateUserDto>,
) -> AppResult<ApiResponse<UserResponse>> {
    let id = parse_ulid(&id)?;
    let user = state
        .user_service
        .update_user(id, payload.name, payload.email)
        .await?;
    Ok(ApiResponse::success(UserResponse::from(user)))
}

#[utoipa::path(
    delete,
    path = "/users/{id}",
    params(("id" = String, Path, description = "User id")),
    responses(
        (status = 200, description = "User deleted", body = ApiResponse<String>)
    ),
    tag = "User"
)]
/// 接口功能：delete_user，删除用户
pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<ApiResponse<String>> {
    let id = parse_ulid(&id)?;
    state.user_service.delete_user(id).await?;
    Ok(ApiResponse::success("deleted".to_string()))
}
