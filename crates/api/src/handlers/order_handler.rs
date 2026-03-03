//! Goods order handlers

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use axum_application::{CreateGoodsOrderInput, OrderPreview, OrderService};
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
/// DTO定义：OrderItemRequest，订单商品项请求参数
pub struct OrderItemRequest {
    /// 参数：product_id，商品唯一标识
    pub product_id: String,
    /// 参数：title_snapshot，商品标题快照
    pub title_snapshot: String,
    /// 参数：price_snapshot，下单时价格快照
    pub price_snapshot: i32,
    /// 参数：qty，商品数量
    pub qty: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：CreateOrderRequest，创建商品订单请求参数
pub struct CreateOrderRequest {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：delivery_type，配送方式
    pub delivery_type: String,
    /// 参数：items，明细项列表
    pub items: Vec<OrderItemRequest>,
    /// 参数：distance_km，距离（公里）
    pub distance_km: Option<f64>,
    /// 参数：address_id，收货地址唯一标识
    pub address_id: Option<String>,
    /// 参数：address_snapshot，收货地址快照
    pub address_snapshot: Option<serde_json::Value>,
    /// 参数：store_snapshot，门店信息快照
    pub store_snapshot: Option<serde_json::Value>,
    /// 参数：remark，备注信息
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：PreviewOrderRequest，订单预览请求参数
pub struct PreviewOrderRequest {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：delivery_type，配送方式
    pub delivery_type: String,
    /// 参数：items，明细项列表
    pub items: Vec<OrderItemRequest>,
    /// 参数：distance_km，距离（公里）
    pub distance_km: Option<f64>,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：PayOrderRequest，商品订单支付请求参数
pub struct PayOrderRequest {
    /// 参数：order_id，订单唯一标识
    pub order_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：CancelOrderRequest，取消商品订单请求参数
pub struct CancelOrderRequest {
    /// 参数：reason，取消原因
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：ListOrdersQuery，商品订单列表查询参数
pub struct ListOrdersQuery {
    /// 参数：status，业务状态
    pub status: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：OrderItemResponse，订单商品项响应数据
pub struct OrderItemResponse {
    /// 参数：product_id，商品唯一标识
    pub product_id: String,
    /// 参数：title_snapshot，商品标题快照
    pub title_snapshot: String,
    /// 参数：price_snapshot，下单时价格快照
    pub price_snapshot: i32,
    /// 参数：qty，商品数量
    pub qty: i32,
}

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：OrderResponse，订单详情响应数据
pub struct OrderResponse {
    /// 参数：order_id，订单唯一标识
    pub order_id: String,
    /// 参数：user_id，用户唯一标识
    pub user_id: String,
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：delivery_type，配送方式
    pub delivery_type: String,
    /// 参数：status，业务状态
    pub status: String,
    /// 参数：pay_status，支付状态
    pub pay_status: String,
    /// 参数：items，明细项列表
    pub items: Vec<OrderItemResponse>,
    /// 参数：amount_goods，商品总金额
    pub amount_goods: i32,
    /// 参数：amount_delivery_fee，配送费金额
    pub amount_delivery_fee: i32,
    /// 参数：amount_discount，优惠金额
    pub amount_discount: i32,
    /// 参数：amount_payable，应付金额
    pub amount_payable: i32,
    /// 参数：distance_km，距离（公里）
    pub distance_km: Option<f64>,
    /// 参数：address_snapshot，收货地址快照
    pub address_snapshot: Option<serde_json::Value>,
    /// 参数：store_snapshot，门店信息快照
    pub store_snapshot: Option<serde_json::Value>,
    /// 参数：remark，备注信息
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：OrderPreviewResponse，订单预览响应数据
pub struct OrderPreviewResponse {
    /// 参数：amount_goods，商品总金额
    pub amount_goods: i32,
    /// 参数：amount_delivery_fee，配送费金额
    pub amount_delivery_fee: i32,
    /// 参数：amount_discount，优惠金额
    pub amount_discount: i32,
    /// 参数：amount_payable，应付金额
    pub amount_payable: i32,
    /// 参数：distance_km，距离（公里）
    pub distance_km: Option<f64>,
    /// 参数：deliverable，是否可配送
    pub deliverable: bool,
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
    state
        .order_service
        .as_ref()
        .cloned()
        .map(|item| (*item).clone())
        .ok_or_else(|| AppError::Internal("order service not initialized".into()))
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
) -> AppResult<ApiResponse<OrderPreviewResponse>> {
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
    headers: HeaderMap,
    Json(payload): Json<CreateOrderRequest>,
) -> AppResult<ApiResponse<OrderResponse>> {
    let user_id = parse_user_id(&headers)?;
    let store_id = parse_ulid(&payload.store_id, "store_id")?;
    let delivery_type = parse_delivery_type(&payload.delivery_type)?;
    let service = get_service(&state)?;
    let mut address_snapshot = payload.address_snapshot;

    if delivery_type == DeliveryType::Delivery && address_snapshot.is_none() {
        let address_service = state
            .address_service
            .as_ref()
            .cloned()
            .map(|item| (*item).clone())
            .ok_or_else(|| AppError::Internal("address service not initialized".into()))?;
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
            address_snapshot,
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
/// 接口功能：pay_order，发起商品订单支付
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
/// 接口功能：list_orders，查询当前用户商品订单列表
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
/// 接口功能：get_order，获取商品订单详情
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
/// 接口功能：cancel_order，取消商品订单
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
