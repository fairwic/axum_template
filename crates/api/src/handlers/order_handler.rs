//! Goods order handlers

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use axum_application::{CreateGoodsOrderInput, OrderService};
use axum_common::{ApiResponse, AppError, AppResult};
use axum_domain::order::entity::{
    DeliveryType, GoodsOrder, GoodsOrderItem, GoodsOrderStatus, PayStatus,
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use utoipa::ToSchema;

use crate::state::AppState;

const USER_ID_HEADER: &str = "x-user-id";

#[derive(Debug, Deserialize, ToSchema)]
pub struct OrderItemRequest {
    pub product_id: String,
    pub title_snapshot: String,
    pub price_snapshot: i32,
    pub qty: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateOrderRequest {
    pub store_id: String,
    pub delivery_type: String,
    pub items: Vec<OrderItemRequest>,
    pub distance_km: Option<f64>,
    pub address_snapshot: Option<serde_json::Value>,
    pub store_snapshot: Option<serde_json::Value>,
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PayOrderRequest {
    pub order_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CancelOrderRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListOrdersQuery {
    pub status: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrderItemResponse {
    pub product_id: String,
    pub title_snapshot: String,
    pub price_snapshot: i32,
    pub qty: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrderResponse {
    pub order_id: String,
    pub user_id: String,
    pub store_id: String,
    pub delivery_type: String,
    pub status: String,
    pub pay_status: String,
    pub items: Vec<OrderItemResponse>,
    pub amount_goods: i32,
    pub amount_delivery_fee: i32,
    pub amount_discount: i32,
    pub amount_payable: i32,
    pub distance_km: Option<f64>,
    pub address_snapshot: Option<serde_json::Value>,
    pub store_snapshot: Option<serde_json::Value>,
    pub remark: Option<String>,
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

fn parse_delivery_type(value: &str) -> AppResult<DeliveryType> {
    match value {
        "DELIVERY" => Ok(DeliveryType::Delivery),
        "PICKUP" => Ok(DeliveryType::Pickup),
        _ => Err(AppError::Validation(
            "delivery_type must be DELIVERY/PICKUP".into(),
        )),
    }
}

fn status_to_string(value: &GoodsOrderStatus) -> &'static str {
    match value {
        GoodsOrderStatus::PendingPay => "PENDING_PAY",
        GoodsOrderStatus::PendingAccept => "PENDING_ACCEPT",
        GoodsOrderStatus::Accepted => "ACCEPTED",
        GoodsOrderStatus::Delivering => "DELIVERING",
        GoodsOrderStatus::WaitingPickup => "WAITING_PICKUP",
        GoodsOrderStatus::Completed => "COMPLETED",
        GoodsOrderStatus::Canceled => "CANCELED",
        GoodsOrderStatus::Closed => "CLOSED",
    }
}

fn pay_status_to_string(value: &PayStatus) -> &'static str {
    match value {
        PayStatus::Unpaid => "UNPAID",
        PayStatus::Paid => "PAID",
        PayStatus::Refunded => "REFUNDED",
    }
}

fn delivery_type_to_string(value: &DeliveryType) -> &'static str {
    match value {
        DeliveryType::Delivery => "DELIVERY",
        DeliveryType::Pickup => "PICKUP",
    }
}

fn to_response(order: GoodsOrder) -> OrderResponse {
    OrderResponse {
        order_id: order.id.to_string(),
        user_id: order.user_id.to_string(),
        store_id: order.store_id.to_string(),
        delivery_type: delivery_type_to_string(&order.delivery_type).to_string(),
        status: status_to_string(&order.status).to_string(),
        pay_status: pay_status_to_string(&order.pay_status).to_string(),
        items: order
            .items
            .into_iter()
            .map(|item| OrderItemResponse {
                product_id: item.product_id.to_string(),
                title_snapshot: item.title_snapshot,
                price_snapshot: item.price_snapshot,
                qty: item.qty,
            })
            .collect(),
        amount_goods: order.amount_goods,
        amount_delivery_fee: order.amount_delivery_fee,
        amount_discount: order.amount_discount,
        amount_payable: order.amount_payable,
        distance_km: order.distance_km,
        address_snapshot: order.address_snapshot,
        store_snapshot: order.store_snapshot,
        remark: order.remark,
    }
}

fn get_service(state: &AppState) -> AppResult<OrderService> {
    state
        .order_service
        .as_ref()
        .cloned()
        .map(|item| (*item).clone())
        .ok_or_else(|| AppError::Internal("order service not initialized".into()))
}

#[utoipa::path(
    post,
    path = "/orders/create",
    request_body = CreateOrderRequest,
    responses((status = 200, body = ApiResponse<OrderResponse>)),
    tag = "Order"
)]
pub async fn create_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateOrderRequest>,
) -> AppResult<ApiResponse<OrderResponse>> {
    let user_id = parse_user_id(&headers)?;
    let store_id = parse_ulid(&payload.store_id, "store_id")?;
    let delivery_type = parse_delivery_type(&payload.delivery_type)?;
    let service = get_service(&state)?;

    let mut items = Vec::with_capacity(payload.items.len());
    for item in payload.items {
        items.push(GoodsOrderItem {
            product_id: parse_ulid(&item.product_id, "product_id")?,
            title_snapshot: item.title_snapshot,
            price_snapshot: item.price_snapshot,
            qty: item.qty,
        });
    }

    let order = service
        .create(CreateGoodsOrderInput {
            user_id,
            store_id,
            delivery_type,
            items,
            distance_km: payload.distance_km,
            address_snapshot: payload.address_snapshot,
            store_snapshot: payload.store_snapshot,
            remark: payload.remark,
        })
        .await?;
    Ok(ApiResponse::success(to_response(order)))
}

#[utoipa::path(
    post,
    path = "/orders/pay",
    request_body = PayOrderRequest,
    responses((status = 200, body = ApiResponse<OrderResponse>)),
    tag = "Order"
)]
pub async fn pay_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<PayOrderRequest>,
) -> AppResult<ApiResponse<OrderResponse>> {
    let user_id = parse_user_id(&headers)?;
    let order_id = parse_ulid(&payload.order_id, "order_id")?;
    let order = get_service(&state)?.pay(user_id, order_id).await?;
    Ok(ApiResponse::success(to_response(order)))
}

#[utoipa::path(
    get,
    path = "/orders",
    params(("status" = Option<String>, Query, description = "Status filter")),
    responses((status = 200, body = ApiResponse<Vec<OrderResponse>>)),
    tag = "Order"
)]
pub async fn list_orders(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListOrdersQuery>,
) -> AppResult<ApiResponse<Vec<OrderResponse>>> {
    let user_id = parse_user_id(&headers)?;
    let service = get_service(&state)?;
    let mut orders = service.list_by_user(user_id).await?;
    if let Some(status) = query.status {
        orders.retain(|item| status_to_string(&item.status) == status);
    }
    Ok(ApiResponse::success(
        orders.into_iter().map(to_response).collect(),
    ))
}

#[utoipa::path(
    get,
    path = "/orders/{id}",
    params(("id" = String, Path, description = "Order ID")),
    responses((status = 200, body = ApiResponse<OrderResponse>)),
    tag = "Order"
)]
pub async fn get_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(order_id): Path<String>,
) -> AppResult<ApiResponse<OrderResponse>> {
    let user_id = parse_user_id(&headers)?;
    let order_id = parse_ulid(&order_id, "order_id")?;
    let order = get_service(&state)?.get_by_user(user_id, order_id).await?;
    Ok(ApiResponse::success(to_response(order)))
}

#[utoipa::path(
    post,
    path = "/orders/{id}/cancel",
    params(("id" = String, Path, description = "Order ID")),
    request_body = CancelOrderRequest,
    responses((status = 200, body = ApiResponse<OrderResponse>)),
    tag = "Order"
)]
pub async fn cancel_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(order_id): Path<String>,
    Json(payload): Json<CancelOrderRequest>,
) -> AppResult<ApiResponse<OrderResponse>> {
    let user_id = parse_user_id(&headers)?;
    let order_id = parse_ulid(&order_id, "order_id")?;
    let order = get_service(&state)?
        .cancel(user_id, order_id, payload.reason)
        .await?;
    Ok(ApiResponse::success(to_response(order)))
}
