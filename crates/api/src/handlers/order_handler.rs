//! Goods order handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use axum_application::{CreateGoodsOrderInput, OrderPreview, OrderService};
use axum_common_api::ApiResponse;
use axum_core_kernel::{AppError, AppResult};
use axum_domain::order::entity::{
    DeliveryType, GoodsOrder, GoodsOrderItem, GoodsOrderStatus, PayStatus,
};
use ulid::Ulid;

use crate::dtos::order_dto::{
    CancelOrderRequest, CreateOrderRequest, ListOrdersQuery, OrderItemRequest, OrderItemResponse,
    OrderPreviewResponse, OrderResponse, PayOrderRequest, PreviewOrderRequest,
};
use crate::extractors::{parse_ulid, AuthUser};
use crate::state::AppState;

fn parse_delivery_type(value: &str) -> AppResult<DeliveryType> {
    match value {
        "DELIVERY" => Ok(DeliveryType::Delivery),
        "PICKUP" => Ok(DeliveryType::Pickup),
        _ => Err(AppError::Validation(
            "delivery_type must be DELIVERY/PICKUP".into(),
        )),
    }
}

/// API DTO -> Application Input 映射
fn map_order_items(items: Vec<OrderItemRequest>) -> AppResult<Vec<GoodsOrderItem>> {
    let mut mapped = Vec::with_capacity(items.len());
    for item in items {
        mapped.push(GoodsOrderItem {
            product_id: parse_ulid(&item.product_id, "product_id")?,
            title_snapshot: item.title_snapshot,
            price_snapshot: item.price_snapshot,
            qty: item.qty,
        });
    }
    Ok(mapped)
}

/// API DTO -> Application Input 映射
fn map_create_order_input(
    payload: CreateOrderRequest,
    user_id: Ulid,
    store_id: Ulid,
    delivery_type: DeliveryType,
    address_snapshot: Option<serde_json::Value>,
) -> AppResult<CreateGoodsOrderInput> {
    Ok(CreateGoodsOrderInput {
        user_id,
        store_id,
        delivery_type,
        items: map_order_items(payload.items)?,
        distance_km: payload.distance_km,
        address_snapshot,
        store_snapshot: payload.store_snapshot,
        remark: payload.remark,
    })
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

fn preview_to_response(preview: OrderPreview) -> OrderPreviewResponse {
    OrderPreviewResponse {
        amount_goods: preview.amount_goods,
        amount_delivery_fee: preview.amount_delivery_fee,
        amount_discount: preview.amount_discount,
        amount_payable: preview.amount_payable,
        distance_km: preview.distance_km,
        deliverable: preview.deliverable,
    }
}

fn get_service(state: &AppState) -> AppResult<OrderService> {
    Ok((**state.order_service_ref()?).clone())
}

#[utoipa::path(
    post,
    path = "/orders/preview",
    request_body = PreviewOrderRequest,
    responses((status = 200, body = ApiResponse<OrderPreviewResponse>)),
    tag = "Order"
)]
/// 接口功能：preview_order，预览订单金额与可配送性
pub async fn preview_order(
    State(state): State<AppState>,
    Json(payload): Json<PreviewOrderRequest>,
) -> crate::error::ApiResult<ApiResponse<OrderPreviewResponse>> {
    let store_id = parse_ulid(&payload.store_id, "store_id")?;
    let delivery_type = parse_delivery_type(&payload.delivery_type)?;
    let service = get_service(&state)?;
    let items = map_order_items(payload.items)?;

    let preview = service
        .preview(store_id, delivery_type, items, payload.distance_km)
        .await?;
    Ok(ApiResponse::success(preview_to_response(preview)))
}

#[utoipa::path(
    post,
    path = "/orders/create",
    request_body = CreateOrderRequest,
    responses((status = 200, body = ApiResponse<OrderResponse>)),
    tag = "Order"
)]
/// 接口功能：create_order，提交并创建商品订单
pub async fn create_order(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateOrderRequest>,
) -> crate::error::ApiResult<ApiResponse<OrderResponse>> {
    let user_id = auth_user.user_id;
    let store_id = parse_ulid(&payload.store_id, "store_id")?;
    let delivery_type = parse_delivery_type(&payload.delivery_type)?;
    let service = get_service(&state)?;
    let mut address_snapshot = payload.address_snapshot.clone();

    if delivery_type == DeliveryType::Delivery && address_snapshot.is_none() {
        let address_service = (**state.address_service_ref()?).clone();
        let address_id = payload
            .address_id
            .as_ref()
            .ok_or_else(|| AppError::Validation("address_id is required".into()))
            .and_then(|id| parse_ulid(id, "address_id"))?;
        let address = address_service.get_by_id(user_id, address_id).await?;
        address_snapshot = Some(serde_json::json!({
            "address_id": address.id.to_string(),
            "name": address.name,
            "phone": address.phone,
            "detail": address.detail,
            "lat": address.lat,
            "lng": address.lng,
        }));
    }
    let input =
        map_create_order_input(payload, user_id, store_id, delivery_type, address_snapshot)?;
    let order = service.create(input).await?;
    Ok(ApiResponse::success(to_response(order)))
}

#[utoipa::path(
    post,
    path = "/orders/pay",
    request_body = PayOrderRequest,
    responses((status = 200, body = ApiResponse<OrderResponse>)),
    tag = "Order"
)]
/// 接口功能：pay_order，发起商品订单支付
pub async fn pay_order(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<PayOrderRequest>,
) -> crate::error::ApiResult<ApiResponse<OrderResponse>> {
    let user_id = auth_user.user_id;
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
/// 接口功能：list_orders，查询当前用户商品订单列表
pub async fn list_orders(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(query): Query<ListOrdersQuery>,
) -> crate::error::ApiResult<ApiResponse<Vec<OrderResponse>>> {
    let user_id = auth_user.user_id;
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
/// 接口功能：get_order，获取商品订单详情
pub async fn get_order(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(order_id): Path<String>,
) -> crate::error::ApiResult<ApiResponse<OrderResponse>> {
    let user_id = auth_user.user_id;
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
/// 接口功能：cancel_order，取消商品订单
pub async fn cancel_order(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(order_id): Path<String>,
    Json(payload): Json<CancelOrderRequest>,
) -> crate::error::ApiResult<ApiResponse<OrderResponse>> {
    let user_id = auth_user.user_id;
    let order_id = parse_ulid(&order_id, "order_id")?;
    let cancel_timeout_secs = state.biz_config.read().await.cancel_timeout_secs;
    let order = get_service(&state)?
        .cancel(user_id, order_id, payload.reason, cancel_timeout_secs)
        .await?;
    Ok(ApiResponse::success(to_response(order)))
}

#[utoipa::path(
    post,
    path = "/orders/{id}/repurchase",
    params(("id" = String, Path, description = "Order ID")),
    responses((status = 200, body = ApiResponse<OrderResponse>)),
    tag = "Order"
)]
/// 接口功能：repurchase_order，基于历史订单再次下单
pub async fn repurchase_order(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(order_id): Path<String>,
) -> crate::error::ApiResult<ApiResponse<OrderResponse>> {
    let user_id = auth_user.user_id;
    let order_id = parse_ulid(&order_id, "order_id")?;
    let order = get_service(&state)?.repurchase(user_id, order_id).await?;
    Ok(ApiResponse::success(to_response(order)))
}
