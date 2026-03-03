//! Category handlers

use axum::extract::{Query, State};
use axum_common::{ApiResponse, AppError, AppResult};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use utoipa::ToSchema;

use axum_domain::category::entity::{Category, CategoryStatus};

use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：CategoryQuery，商品分类查询参数
pub struct CategoryQuery {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：CategoryResponse，商品分类响应数据
pub struct CategoryResponse {
    /// 参数：id，记录唯一标识
    pub id: String,
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：name，名称
    pub name: String,
    /// 参数：sort_order，排序序号
    pub sort_order: i32,
    /// 参数：status，业务状态
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
/// 接口功能：list_categories，获取门店商品分类列表
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
