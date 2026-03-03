//! Admin product handlers

use axum::{
    extract::{Path, State},
    Json,
};
use axum_application::{CreateProductInput, ProductService, UpdateProductInput};
use axum_common::{ApiResponse, AppError, AppResult};
use axum_domain::product::entity::{Product, ProductStatus};

use crate::dtos::product_dto::{
    AdminCreateProductRequest, AdminUpdateProductRequest, ProductResponse,
};
use crate::extractors::parse_ulid;
use crate::state::AppState;

fn parse_product_status(value: &str) -> AppResult<ProductStatus> {
    match value {
        "ON" => Ok(ProductStatus::On),
        "OFF" => Ok(ProductStatus::Off),
        _ => Err(AppError::Validation("invalid product status".into())),
    }
}

fn map_create_product_input(payload: AdminCreateProductRequest) -> AppResult<CreateProductInput> {
    Ok(CreateProductInput {
        store_id: parse_ulid(&payload.store_id, "store_id")?,
        category_id: parse_ulid(&payload.category_id, "category_id")?,
        title: payload.title,
        subtitle: payload.subtitle,
        cover_image: payload.cover_image,
        images: payload.images,
        price: payload.price,
        original_price: payload.original_price,
        stock: payload.stock,
        status: parse_product_status(&payload.status)?,
        tags: payload.tags,
    })
}

fn map_update_product_input(payload: AdminUpdateProductRequest) -> AppResult<UpdateProductInput> {
    Ok(UpdateProductInput {
        store_id: parse_ulid(&payload.store_id, "store_id")?,
        category_id: parse_ulid(&payload.category_id, "category_id")?,
        title: payload.title,
        subtitle: payload.subtitle,
        cover_image: payload.cover_image,
        images: payload.images,
        price: payload.price,
        original_price: payload.original_price,
        stock: payload.stock,
        status: parse_product_status(&payload.status)?,
        tags: payload.tags,
    })
}

fn status_to_string(value: &ProductStatus) -> &'static str {
    match value {
        ProductStatus::On => "ON",
        ProductStatus::Off => "OFF",
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
        status: status_to_string(&product.status).to_string(),
        tags: product.tags,
    }
}

fn get_service(state: &AppState) -> ProductService {
    (*state.product_service).clone()
}

#[utoipa::path(
    post,
    path = "/admin/products",
    request_body = AdminCreateProductRequest,
    responses((status = 200, body = ApiResponse<ProductResponse>)),
    tag = "AdminProduct"
)]
/// 接口功能：admin_create_product，后台创建商品
pub async fn admin_create_product(
    State(state): State<AppState>,
    Json(payload): Json<AdminCreateProductRequest>,
) -> AppResult<ApiResponse<ProductResponse>> {
    let input = map_create_product_input(payload)?;
    let product = get_service(&state).admin_create(input).await?;
    Ok(ApiResponse::success(to_response(product)))
}

#[utoipa::path(
    put,
    path = "/admin/products/{id}",
    params(("id" = String, Path, description = "Product ID")),
    request_body = AdminUpdateProductRequest,
    responses((status = 200, body = ApiResponse<ProductResponse>)),
    tag = "AdminProduct"
)]
/// 接口功能：admin_update_product，后台更新商品
pub async fn admin_update_product(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<AdminUpdateProductRequest>,
) -> AppResult<ApiResponse<ProductResponse>> {
    let product_id = parse_ulid(&id, "product_id")?;
    let input = map_update_product_input(payload)?;
    let product = get_service(&state).admin_update(product_id, input).await?;
    Ok(ApiResponse::success(to_response(product)))
}
