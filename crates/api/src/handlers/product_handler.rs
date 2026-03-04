//! Product handlers

use axum::extract::{Path, Query, State};
use axum_api_common::{ApiResponse, PagedResponse};
use axum_core_kernel::AppError;
use ulid::Ulid;

use axum_domain::product::entity::{Product, ProductStatus};

use crate::dtos::product_dto::{
    ProductDetailQuery, ProductListQuery, ProductResponse, ProductSearchQuery,
};
use crate::state::AppState;

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
/// 接口功能：list_products，分页查询门店商品列表
pub async fn list_products(
    State(state): State<AppState>,
    Query(query): Query<ProductListQuery>,
) -> crate::error::ApiResult<ApiResponse<PagedResponse<ProductResponse>>> {
    let store_id = Ulid::from_string(&query.store_id)
        .map_err(|_| AppError::Validation("invalid store_id".into()))?;
    let category_id = Ulid::from_string(&query.category_id)
        .map_err(|_| AppError::Validation("invalid category_id".into()))?;

    let page = state
        .product_service
        .list_by_category(store_id, category_id, query.page, query.page_size)
        .await?;

    let data = PagedResponse {
        items: page.items.into_iter().map(to_response).collect(),
        total: page.total,
        page: page.page,
        page_size: page.page_size,
        total_pages: page.total_pages,
    };
    Ok(ApiResponse::success(data))
}

#[utoipa::path(
    get,
    path = "/products/search",
    params(("store_id" = String, Query), ("keyword" = String, Query), ("page" = i64, Query), ("page_size" = i64, Query)),
    responses((status = 200, description = "Product search", body = ApiResponse<PagedResponse<ProductResponse>>)),
    tag = "Product"
)]
/// 接口功能：search_products，按关键词搜索门店商品
pub async fn search_products(
    State(state): State<AppState>,
    Query(query): Query<ProductSearchQuery>,
) -> crate::error::ApiResult<ApiResponse<PagedResponse<ProductResponse>>> {
    let store_id = Ulid::from_string(&query.store_id)
        .map_err(|_| AppError::Validation("invalid store_id".into()))?;

    let page = state
        .product_service
        .search(store_id, &query.keyword, query.page, query.page_size)
        .await?;

    let data = PagedResponse {
        items: page.items.into_iter().map(to_response).collect(),
        total: page.total,
        page: page.page,
        page_size: page.page_size,
        total_pages: page.total_pages,
    };
    Ok(ApiResponse::success(data))
}

#[utoipa::path(
    get,
    path = "/products/{id}",
    params(("id" = String, Path), ("store_id" = String, Query)),
    responses((status = 200, description = "Product detail", body = ApiResponse<ProductResponse>)),
    tag = "Product"
)]
/// 接口功能：get_product，获取商品详情
pub async fn get_product(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
    Query(query): Query<ProductDetailQuery>,
) -> crate::error::ApiResult<ApiResponse<ProductResponse>> {
    let store_id = Ulid::from_string(&query.store_id)
        .map_err(|_| AppError::Validation("invalid store_id".into()))?;
    let product_id =
        Ulid::from_string(&product_id).map_err(|_| AppError::Validation("invalid id".into()))?;
    let product = state
        .product_service
        .get_by_id(store_id, product_id)
        .await?;
    Ok(ApiResponse::success(to_response(product)))
}
