//! Category handlers

use axum::extract::{Query, State};
use axum_api_common::ApiResponse;
use axum_core_kernel::AppError;
use ulid::Ulid;

use axum_domain::category::entity::{Category, CategoryStatus};

use crate::dtos::category_dto::{CategoryQuery, CategoryResponse};
use crate::state::AppState;

fn status_to_string(status: &CategoryStatus) -> String {
    match status {
        CategoryStatus::On => "ON".to_string(),
        CategoryStatus::Off => "OFF".to_string(),
    }
}

fn to_response(category: Category) -> CategoryResponse {
    CategoryResponse {
        id: category.id.to_string(),
        store_id: category.store_id.to_string(),
        name: category.name,
        sort_order: category.sort_order,
        status: status_to_string(&category.status),
    }
}

#[utoipa::path(
    get,
    path = "/categories",
    params(("store_id" = String, Query, description = "Store id")),
    responses((status = 200, description = "Category list", body = ApiResponse<Vec<CategoryResponse>>)),
    tag = "Category"
)]
/// 接口功能：list_categories，获取门店商品分类列表
pub async fn list_categories(
    State(state): State<AppState>,
    Query(query): Query<CategoryQuery>,
) -> crate::error::ApiResult<ApiResponse<Vec<CategoryResponse>>> {
    let store_id = Ulid::from_string(&query.store_id)
        .map_err(|_| AppError::Validation("invalid store_id".into()))?;
    let categories = state.category_service.list_by_store(store_id).await?;
    let data = categories.into_iter().map(to_response).collect();
    Ok(ApiResponse::success(data))
}
