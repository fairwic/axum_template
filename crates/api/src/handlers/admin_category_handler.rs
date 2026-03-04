//! Admin category handlers

use axum::{
    extract::{Path, State},
    Json,
};
use axum_application::{CategoryService, CreateCategoryInput, UpdateCategoryInput};
use axum_api_common::ApiResponse;
use axum_core_kernel::{AppError, AppResult};
use axum_domain::category::entity::{Category, CategoryStatus};

use crate::dtos::category_dto::{
    AdminCreateCategoryRequest, AdminUpdateCategoryRequest, CategoryResponse,
};
use crate::extractors::parse_ulid;
use crate::state::AppState;

fn parse_category_status(value: &str) -> AppResult<CategoryStatus> {
    match value {
        "ON" => Ok(CategoryStatus::On),
        "OFF" => Ok(CategoryStatus::Off),
        _ => Err(AppError::Validation("invalid category status".into())),
    }
}

fn map_create_category_input(
    payload: AdminCreateCategoryRequest,
) -> AppResult<CreateCategoryInput> {
    Ok(CreateCategoryInput {
        store_id: parse_ulid(&payload.store_id, "store_id")?,
        name: payload.name,
        sort_order: payload.sort_order,
        status: parse_category_status(&payload.status)?,
    })
}

fn map_update_category_input(
    payload: AdminUpdateCategoryRequest,
) -> AppResult<UpdateCategoryInput> {
    Ok(UpdateCategoryInput {
        name: payload.name,
        sort_order: payload.sort_order,
        status: parse_category_status(&payload.status)?,
    })
}

fn status_to_string(value: &CategoryStatus) -> &'static str {
    match value {
        CategoryStatus::On => "ON",
        CategoryStatus::Off => "OFF",
    }
}

fn to_response(category: Category) -> CategoryResponse {
    CategoryResponse {
        id: category.id.to_string(),
        store_id: category.store_id.to_string(),
        name: category.name,
        sort_order: category.sort_order,
        status: status_to_string(&category.status).to_string(),
    }
}

fn get_service(state: &AppState) -> CategoryService {
    (*state.category_service).clone()
}

#[utoipa::path(
    post,
    path = "/admin/categories",
    request_body = AdminCreateCategoryRequest,
    responses((status = 200, body = ApiResponse<CategoryResponse>)),
    tag = "AdminCategory"
)]
/// 接口功能：admin_create_category，后台创建类目
pub async fn admin_create_category(
    State(state): State<AppState>,
    Json(payload): Json<AdminCreateCategoryRequest>,
) -> crate::error::ApiResult<ApiResponse<CategoryResponse>> {
    let input = map_create_category_input(payload)?;
    let category = get_service(&state).admin_create(input).await?;
    Ok(ApiResponse::success(to_response(category)))
}

#[utoipa::path(
    put,
    path = "/admin/categories/{id}",
    params(("id" = String, Path, description = "Category ID")),
    request_body = AdminUpdateCategoryRequest,
    responses((status = 200, body = ApiResponse<CategoryResponse>)),
    tag = "AdminCategory"
)]
/// 接口功能：admin_update_category，后台更新类目
pub async fn admin_update_category(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<AdminUpdateCategoryRequest>,
) -> crate::error::ApiResult<ApiResponse<CategoryResponse>> {
    let category_id = parse_ulid(&id, "category_id")?;
    let input = map_update_category_input(payload)?;
    let category = get_service(&state).admin_update(category_id, input).await?;
    Ok(ApiResponse::success(to_response(category)))
}
