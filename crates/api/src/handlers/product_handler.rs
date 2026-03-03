//! Product handlers

use axum::extract::{Path, Query, State};
use axum_common::{ApiResponse, AppError, AppResult, PagedResponse};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use utoipa::ToSchema;

use axum_domain::product::entity::{Product, ProductStatus};

use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：ProductListQuery，商品列表查询参数
pub struct ProductListQuery {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：category_id，分类唯一标识
    pub category_id: String,
    #[serde(default = "default_page")]
    /// 参数：page，页码
    pub page: i64,
    #[serde(default = "default_page_size")]
    /// 参数：page_size，每页条数
    pub page_size: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：ProductSearchQuery，商品搜索查询参数
pub struct ProductSearchQuery {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：keyword，搜索关键词
    pub keyword: String,
    #[serde(default = "default_page")]
    /// 参数：page，页码
    pub page: i64,
    #[serde(default = "default_page_size")]
    /// 参数：page_size，每页条数
    pub page_size: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：ProductDetailQuery，商品详情查询参数
pub struct ProductDetailQuery {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    20
}

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：ProductResponse，商品响应数据
pub struct ProductResponse {
    /// 参数：id，记录唯一标识
    pub id: String,
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：category_id，分类唯一标识
    pub category_id: String,
    /// 参数：title，商品标题
    pub title: String,
    /// 参数：subtitle，商品副标题
    pub subtitle: Option<String>,
    /// 参数：cover_image，封面图 URL
    pub cover_image: String,
    /// 参数：images，商品图片列表
    pub images: Vec<String>,
    /// 参数：price，当前售价
    pub price: i32,
    /// 参数：original_price，商品原价
    pub original_price: Option<i32>,
    /// 参数：stock，库存数量
    pub stock: i32,
    /// 参数：status，业务状态
    pub status: String,
    /// 参数：tags，标签列表
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
/// 接口功能：list_products，分页查询门店商品列表
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
/// 接口功能：search_products，按关键词搜索门店商品
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
) -> AppResult<ApiResponse<ProductResponse>> {
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
