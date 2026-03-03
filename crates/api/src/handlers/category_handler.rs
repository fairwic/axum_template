//! Category handlers

use axum::{extract::{Query, State}, Json};
use axum_common::{ApiResponse, AppError, AppResult};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use ulid::Ulid;

use axum_domain::category::entity::{Category, CategoryStatus};

use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CategoryQuery {
    pub store_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CategoryResponse {
    pub id: String,
    pub store_id: String,
    pub name: String,
    pub sort_order: i32,
    pub status: String,
}

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
pub async fn list_categories(
    State(state): State<AppState>,
    Query(query): Query<CategoryQuery>,
) -> AppResult<ApiResponse<Vec<CategoryResponse>>> {
    let store_id = Ulid::from_string(&query.store_id)
        .map_err(|_| AppError::Validation("invalid store_id".into()))?;
    let categories = state.category_service.list_by_store(store_id).await?;
    let data = categories.into_iter().map(to_response).collect();
    Ok(ApiResponse::success(data))
}
