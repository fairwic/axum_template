//! Product handlers

use axum::extract::{Query, State};
use axum_common::{ApiResponse, AppError, AppResult, PagedResponse};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use utoipa::ToSchema;

use axum_domain::product::entity::{Product, ProductStatus};

use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ProductListQuery {
    pub store_id: String,
    pub category_id: String,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ProductSearchQuery {
    pub store_id: String,
    pub keyword: String,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    20
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProductResponse {
    pub id: String,
    pub store_id: String,
    pub category_id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub cover_image: String,
    pub images: Vec<String>,
    pub price: i32,
    pub original_price: Option<i32>,
    pub stock: i32,
    pub status: String,
    pub tags: Vec<String>,
}

fn status_to_string(status: &ProductStatus) -> String {
    match status {
        ProductStatus::On => "ON".to_string(),
        ProductStatus::Off => "OFF".to_string(),
    }
}

fn to_response(product: Product) -> ProductResponse {
    ProductResponse {
        id: product.id.to_string(),
        store_id: product.store_id.to_string(),
        category_id: product.category_id.to_string(),
        title: product.title,
        subtitle: product.subtitle,
        cover_image: product.cover_image,
        images: product.images,
        price: product.price,
        original_price: product.original_price,
        stock: product.stock,
        status: status_to_string(&product.status),
        tags: product.tags,
    }
}

#[utoipa::path(
    get,
    path = "/products",
    params(("store_id" = String, Query), ("category_id" = String, Query), ("page" = i64, Query), ("page_size" = i64, Query)),
    responses((status = 200, description = "Product list", body = ApiResponse<PagedResponse<ProductResponse>>)),
    tag = "Product"
)]
pub async fn list_products(
    State(state): State<AppState>,
    Query(query): Query<ProductListQuery>,
) -> AppResult<ApiResponse<PagedResponse<ProductResponse>>> {
    let store_id = Ulid::from_string(&query.store_id)
        .map_err(|_| AppError::Validation("invalid store_id".into()))?;
    let category_id = Ulid::from_string(&query.category_id)
        .map_err(|_| AppError::Validation("invalid category_id".into()))?;

    let page = state
        .product_service
        .list_by_category(store_id, category_id, query.page, query.page_size)
        .await?;

    let data = PagedResponse::new(
        page.items.into_iter().map(to_response).collect(),
        page.total,
        page.page,
        page.page_size,
    );
    Ok(ApiResponse::success(data))
}

#[utoipa::path(
    get,
    path = "/products/search",
    params(("store_id" = String, Query), ("keyword" = String, Query), ("page" = i64, Query), ("page_size" = i64, Query)),
    responses((status = 200, description = "Product search", body = ApiResponse<PagedResponse<ProductResponse>>)),
    tag = "Product"
)]
pub async fn search_products(
    State(state): State<AppState>,
    Query(query): Query<ProductSearchQuery>,
) -> AppResult<ApiResponse<PagedResponse<ProductResponse>>> {
    let store_id = Ulid::from_string(&query.store_id)
        .map_err(|_| AppError::Validation("invalid store_id".into()))?;

    let page = state
        .product_service
        .search(store_id, &query.keyword, query.page, query.page_size)
        .await?;

    let data = PagedResponse::new(
        page.items.into_iter().map(to_response).collect(),
        page.total,
        page.page,
        page.page_size,
    );
    Ok(ApiResponse::success(data))
}
