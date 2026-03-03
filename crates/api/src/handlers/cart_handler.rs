//! Cart handlers

use axum::{
    extract::{Query, State},
    http::HeaderMap,
    Json,
};
use axum_common::{ApiResponse, AppError, AppResult};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use utoipa::ToSchema;

use axum_domain::Cart;

use crate::state::AppState;

const USER_ID_HEADER: &str = "x-user-id";

#[derive(Debug, Deserialize, ToSchema)]
pub struct CartQuery {
    pub store_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddItemRequest {
    pub store_id: String,
    pub product_id: String,
    pub qty: i32,
    pub price_snapshot: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateQtyRequest {
    pub store_id: String,
    pub product_id: String,
    pub qty: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RemoveItemRequest {
    pub store_id: String,
    pub product_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ClearCartRequest {
    pub store_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CartItemResponse {
    pub product_id: String,
    pub qty: i32,
    pub price_snapshot: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CartResponse {
    pub cart_id: String,
    pub store_id: String,
    pub items: Vec<CartItemResponse>,
}

fn parse_ulid(value: &str, field: &str) -> AppResult<Ulid> {
    Ulid::from_string(value).map_err(|_| AppError::Validation(format!("invalid {}", field)))
}

fn parse_user_id(headers: &HeaderMap) -> AppResult<Ulid> {
    let value = headers
        .get(USER_ID_HEADER)
        .ok_or_else(|| AppError::Validation("missing x-user-id".into()))?
        .to_str()
        .map_err(|_| AppError::Validation("invalid x-user-id".into()))?;
    parse_ulid(value, "user_id")
}

fn ensure_positive_qty(qty: i32) -> AppResult<()> {
    if qty <= 0 {
        return Err(AppError::Validation("qty must be greater than 0".into()));
    }
    Ok(())
}

fn to_response(cart: Cart) -> CartResponse {
    CartResponse {
        cart_id: cart.id.to_string(),
        store_id: cart.store_id.to_string(),
        items: cart
            .items
            .into_iter()
            .map(|item| CartItemResponse {
                product_id: item.product_id.to_string(),
                qty: item.qty,
                price_snapshot: item.price_snapshot,
            })
            .collect(),
    }
}

#[utoipa::path(
    get,
    path = "/cart",
    params(("store_id" = String, Query, description = "Store ID")),
    responses((status = 200, description = "Get cart", body = ApiResponse<CartResponse>)),
    tag = "Cart"
)]
pub async fn get_cart(
    State(state): State<AppState>,
    Query(query): Query<CartQuery>,
    headers: HeaderMap,
) -> AppResult<ApiResponse<CartResponse>> {
    let user_id = parse_user_id(&headers)?;
    let store_id = parse_ulid(&query.store_id, "store_id")?;
    let cart = state.cart_service.get_cart(user_id, store_id).await?;
    Ok(ApiResponse::success(to_response(cart)))
}

#[utoipa::path(
    post,
    path = "/cart/add",
    request_body = AddItemRequest,
    responses((status = 200, description = "Add item", body = ApiResponse<CartResponse>)),
    tag = "Cart"
)]
pub async fn add_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AddItemRequest>,
) -> AppResult<ApiResponse<CartResponse>> {
    let user_id = parse_user_id(&headers)?;
    let store_id = parse_ulid(&payload.store_id, "store_id")?;
    let product_id = parse_ulid(&payload.product_id, "product_id")?;
    ensure_positive_qty(payload.qty)?;

    state
        .cart_service
        .add_item(
            user_id,
            store_id,
            product_id,
            payload.qty,
            payload.price_snapshot,
        )
        .await?;
    let cart = state.cart_service.get_cart(user_id, store_id).await?;
    Ok(ApiResponse::success(to_response(cart)))
}

#[utoipa::path(
    post,
    path = "/cart/update_qty",
    request_body = UpdateQtyRequest,
    responses((status = 200, description = "Update qty", body = ApiResponse<CartResponse>)),
    tag = "Cart"
)]
pub async fn update_qty(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateQtyRequest>,
) -> AppResult<ApiResponse<CartResponse>> {
    let user_id = parse_user_id(&headers)?;
    let store_id = parse_ulid(&payload.store_id, "store_id")?;
    let product_id = parse_ulid(&payload.product_id, "product_id")?;
    ensure_positive_qty(payload.qty)?;

    let cart = state.cart_service.get_cart(user_id, store_id).await?;
    let item = cart
        .items
        .iter()
        .find(|item| item.product_id == product_id)
        .ok_or_else(|| AppError::NotFound("cart item not found".into()))?;

    state
        .cart_service
        .add_item(
            user_id,
            store_id,
            product_id,
            payload.qty,
            item.price_snapshot,
        )
        .await?;
    let cart = state.cart_service.get_cart(user_id, store_id).await?;
    Ok(ApiResponse::success(to_response(cart)))
}

#[utoipa::path(
    post,
    path = "/cart/remove",
    request_body = RemoveItemRequest,
    responses((status = 200, description = "Remove item", body = ApiResponse<CartResponse>)),
    tag = "Cart"
)]
pub async fn remove_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<RemoveItemRequest>,
) -> AppResult<ApiResponse<CartResponse>> {
    let user_id = parse_user_id(&headers)?;
    let store_id = parse_ulid(&payload.store_id, "store_id")?;
    let product_id = parse_ulid(&payload.product_id, "product_id")?;

    state
        .cart_service
        .remove_item(user_id, store_id, product_id)
        .await?;
    let cart = state.cart_service.get_cart(user_id, store_id).await?;
    Ok(ApiResponse::success(to_response(cart)))
}

#[utoipa::path(
    post,
    path = "/cart/clear",
    request_body = ClearCartRequest,
    responses((status = 200, description = "Clear cart", body = ApiResponse<CartResponse>)),
    tag = "Cart"
)]
pub async fn clear_cart(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ClearCartRequest>,
) -> AppResult<ApiResponse<CartResponse>> {
    let user_id = parse_user_id(&headers)?;
    let store_id = parse_ulid(&payload.store_id, "store_id")?;

    state.cart_service.clear(user_id, store_id).await?;
    let cart = state.cart_service.get_cart(user_id, store_id).await?;
    Ok(ApiResponse::success(to_response(cart)))
}
