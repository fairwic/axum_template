//! Admin goods order handlers

use axum::extract::{Path, Query, State};
use axum_application::OrderService;
use axum_common::{ApiResponse, AppError, AppResult};
use serde::Deserialize;
use utoipa::ToSchema;
use ulid::Ulid;

use crate::handlers::order_handler::OrderResponse;
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminListOrdersQuery {
    pub store_id: String,
    pub status: Option<String>,
}

fn parse_ulid(value: &str, field: &str) -> AppResult<Ulid> {
    Ulid::from_string(value).map_err(|_| AppError::Validation(format!("invalid {}", field)))
}

fn get_service(state: &AppState) -> AppResult<OrderService> {
    state
        .order_service
        .as_ref()
        .cloned()
        .map(|item| (*item).clone())
        .ok_or_else(|| AppError::Internal("order service not initialized".into()))
}

fn to_response(order: axum_domain::GoodsOrder) -> OrderResponse {
    crate::handlers::order_handler::OrderResponse {
        order_id: order.id.to_string(),
        user_id: order.user_id.to_string(),
        store_id: order.store_id.to_string(),
        delivery_type: match order.delivery_type {
            axum_domain::DeliveryType::Delivery => "DELIVERY".into(),
            axum_domain::DeliveryType::Pickup => "PICKUP".into(),
        },
        status: match order.status {
            axum_domain::GoodsOrderStatus::PendingPay => "PENDING_PAY".into(),
            axum_domain::GoodsOrderStatus::PendingAccept => "PENDING_ACCEPT".into(),
            axum_domain::GoodsOrderStatus::Accepted => "ACCEPTED".into(),
            axum_domain::GoodsOrderStatus::Delivering => "DELIVERING".into(),
            axum_domain::GoodsOrderStatus::WaitingPickup => "WAITING_PICKUP".into(),
            axum_domain::GoodsOrderStatus::Completed => "COMPLETED".into(),
            axum_domain::GoodsOrderStatus::Canceled => "CANCELED".into(),
            axum_domain::GoodsOrderStatus::Closed => "CLOSED".into(),
        },
        pay_status: match order.pay_status {
            axum_domain::PayStatus::Unpaid => "UNPAID".into(),
            axum_domain::PayStatus::Paid => "PAID".into(),
            axum_domain::PayStatus::Refunded => "REFUNDED".into(),
        },
        items: order
            .items
            .into_iter()
            .map(|item| crate::handlers::order_handler::OrderItemResponse {
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

#[utoipa::path(
    get,
    path = "/admin/orders",
    params(
        ("store_id" = String, Query, description = "Store ID"),
        ("status" = Option<String>, Query, description = "Status filter")
    ),
    responses((status = 200, body = ApiResponse<Vec<OrderResponse>>)),
    tag = "AdminOrder"
)]
pub async fn admin_list_orders(
    State(state): State<AppState>,
    Query(query): Query<AdminListOrdersQuery>,
) -> AppResult<ApiResponse<Vec<OrderResponse>>> {
    let store_id = parse_ulid(&query.store_id, "store_id")?;
    let mut orders = get_service(&state)?.admin_list_by_store(store_id).await?;
    if let Some(status) = query.status {
        orders.retain(|item| {
            let value = match item.status {
                axum_domain::GoodsOrderStatus::PendingPay => "PENDING_PAY",
                axum_domain::GoodsOrderStatus::PendingAccept => "PENDING_ACCEPT",
                axum_domain::GoodsOrderStatus::Accepted => "ACCEPTED",
                axum_domain::GoodsOrderStatus::Delivering => "DELIVERING",
                axum_domain::GoodsOrderStatus::WaitingPickup => "WAITING_PICKUP",
                axum_domain::GoodsOrderStatus::Completed => "COMPLETED",
                axum_domain::GoodsOrderStatus::Canceled => "CANCELED",
                axum_domain::GoodsOrderStatus::Closed => "CLOSED",
            };
            value == status
        });
    }

    Ok(ApiResponse::success(
        orders.into_iter().map(to_response).collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/admin/orders/{id}/accept",
    params(("id" = String, Path, description = "Order ID")),
    responses((status = 200, body = ApiResponse<OrderResponse>)),
    tag = "AdminOrder"
)]
pub async fn admin_accept_order(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> AppResult<ApiResponse<OrderResponse>> {
    let order_id = parse_ulid(&order_id, "order_id")?;
    let order = get_service(&state)?.admin_accept(order_id).await?;
    Ok(ApiResponse::success(to_response(order)))
}

#[utoipa::path(
    post,
    path = "/admin/orders/{id}/dispatch",
    params(("id" = String, Path, description = "Order ID")),
    responses((status = 200, body = ApiResponse<OrderResponse>)),
    tag = "AdminOrder"
)]
pub async fn admin_dispatch_order(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> AppResult<ApiResponse<OrderResponse>> {
    let order_id = parse_ulid(&order_id, "order_id")?;
    let order = get_service(&state)?.admin_dispatch(order_id).await?;
    Ok(ApiResponse::success(to_response(order)))
}

#[utoipa::path(
    post,
    path = "/admin/orders/{id}/complete",
    params(("id" = String, Path, description = "Order ID")),
    responses((status = 200, body = ApiResponse<OrderResponse>)),
    tag = "AdminOrder"
)]
pub async fn admin_complete_order(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> AppResult<ApiResponse<OrderResponse>> {
    let order_id = parse_ulid(&order_id, "order_id")?;
    let order = get_service(&state)?.admin_complete(order_id).await?;
    Ok(ApiResponse::success(to_response(order)))
}
