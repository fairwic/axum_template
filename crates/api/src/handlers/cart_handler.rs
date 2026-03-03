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
/// DTO定义：CartQuery，购物车查询参数
pub struct CartQuery {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：AddItemRequest，添加购物车商品请求参数
pub struct AddItemRequest {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：product_id，商品唯一标识
    pub product_id: String,
    /// 参数：qty，商品数量
    pub qty: i32,
    /// 参数：price_snapshot，下单时价格快照
    pub price_snapshot: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：UpdateQtyRequest，更新购物车数量请求参数
pub struct UpdateQtyRequest {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：product_id，商品唯一标识
    pub product_id: String,
    /// 参数：qty，商品数量
    pub qty: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：RemoveItemRequest，移除购物车商品请求参数
pub struct RemoveItemRequest {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：product_id，商品唯一标识
    pub product_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：ClearCartRequest，清空购物车请求参数
pub struct ClearCartRequest {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：CartItemResponse，购物车商品项响应数据
pub struct CartItemResponse {
    /// 参数：product_id，商品唯一标识
    pub product_id: String,
    /// 参数：qty，商品数量
    pub qty: i32,
    /// 参数：price_snapshot，下单时价格快照
    pub price_snapshot: i32,
}

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：CartResponse，购物车响应数据
pub struct CartResponse {
    /// 参数：cart_id，购物车唯一标识
    pub cart_id: String,
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：items，明细项列表
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
/// 接口功能：get_cart，获取指定门店购物车
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
/// 接口功能：add_item，添加商品到购物车
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
/// 接口功能：update_qty，更新购物车商品数量
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
/// 接口功能：remove_item，从购物车移除商品
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
/// 接口功能：clear_cart，清空指定门店购物车
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
