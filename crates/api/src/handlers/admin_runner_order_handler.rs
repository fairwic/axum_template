//! Admin runner order handlers

use axum::extract::{Path, Query, State};
use axum_application::RunnerOrderService;
use axum_common::{ApiResponse, AppError, AppResult};
use serde::Deserialize;
use utoipa::ToSchema;
use ulid::Ulid;

use crate::handlers::runner_order_handler::RunnerOrderResponse;
use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminListRunnerOrdersQuery {
    pub store_id: String,
    pub status: Option<String>,
}

fn parse_ulid(value: &str, field: &str) -> AppResult<Ulid> {
    Ulid::from_string(value).map_err(|_| AppError::Validation(format!("invalid {}", field)))
}

fn get_service(state: &AppState) -> AppResult<RunnerOrderService> {
    state
        .runner_order_service
        .as_ref()
        .cloned()
        .map(|item| (*item).clone())
        .ok_or_else(|| AppError::Internal("runner order service not initialized".into()))
}

fn to_response(order: axum_domain::RunnerOrder) -> RunnerOrderResponse {
    crate::handlers::runner_order_handler::RunnerOrderResponse {
        runner_order_id: order.id.to_string(),
        user_id: order.user_id.to_string(),
        store_id: order.store_id.to_string(),
        status: match order.status {
            axum_domain::RunnerOrderStatus::PendingPay => "PENDING_PAY".into(),
            axum_domain::RunnerOrderStatus::PendingAccept => "PENDING_ACCEPT".into(),
            axum_domain::RunnerOrderStatus::Processing => "PROCESSING".into(),
            axum_domain::RunnerOrderStatus::Delivered => "DELIVERED".into(),
            axum_domain::RunnerOrderStatus::Completed => "COMPLETED".into(),
            axum_domain::RunnerOrderStatus::Canceled => "CANCELED".into(),
            axum_domain::RunnerOrderStatus::Closed => "CLOSED".into(),
        },
        pay_status: match order.pay_status {
            axum_domain::PayStatus::Unpaid => "UNPAID".into(),
            axum_domain::PayStatus::Paid => "PAID".into(),
            axum_domain::PayStatus::Refunded => "REFUNDED".into(),
        },
        express_company: order.express_company,
        pickup_code: order.pickup_code,
        delivery_address: order.delivery_address,
        receiver_name: order.receiver_name,
        receiver_phone: order.receiver_phone,
        remark: order.remark,
        service_fee: order.service_fee,
        distance_km: order.distance_km,
        amount_payable: order.amount_payable,
    }
}

#[utoipa::path(
    get,
    path = "/admin/runner_orders",
    params(
        ("store_id" = String, Query, description = "Store ID"),
        ("status" = Option<String>, Query, description = "Status filter")
    ),
    responses((status = 200, body = ApiResponse<Vec<RunnerOrderResponse>>)),
    tag = "AdminRunnerOrder"
)]
pub async fn admin_list_runner_orders(
    State(state): State<AppState>,
    Query(query): Query<AdminListRunnerOrdersQuery>,
) -> AppResult<ApiResponse<Vec<RunnerOrderResponse>>> {
    let store_id = parse_ulid(&query.store_id, "store_id")?;
    let mut orders = get_service(&state)?.admin_list_by_store(store_id).await?;
    if let Some(status) = query.status {
        orders.retain(|item| {
            let value = match item.status {
                axum_domain::RunnerOrderStatus::PendingPay => "PENDING_PAY",
                axum_domain::RunnerOrderStatus::PendingAccept => "PENDING_ACCEPT",
                axum_domain::RunnerOrderStatus::Processing => "PROCESSING",
                axum_domain::RunnerOrderStatus::Delivered => "DELIVERED",
                axum_domain::RunnerOrderStatus::Completed => "COMPLETED",
                axum_domain::RunnerOrderStatus::Canceled => "CANCELED",
                axum_domain::RunnerOrderStatus::Closed => "CLOSED",
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
    path = "/admin/runner_orders/{id}/accept",
    params(("id" = String, Path, description = "Runner Order ID")),
    responses((status = 200, body = ApiResponse<RunnerOrderResponse>)),
    tag = "AdminRunnerOrder"
)]
pub async fn admin_accept_runner_order(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> AppResult<ApiResponse<RunnerOrderResponse>> {
    let order_id = parse_ulid(&order_id, "runner_order_id")?;
    let order = get_service(&state)?.admin_accept(order_id).await?;
    Ok(ApiResponse::success(to_response(order)))
}

#[utoipa::path(
    post,
    path = "/admin/runner_orders/{id}/delivered",
    params(("id" = String, Path, description = "Runner Order ID")),
    responses((status = 200, body = ApiResponse<RunnerOrderResponse>)),
    tag = "AdminRunnerOrder"
)]
pub async fn admin_delivered_runner_order(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> AppResult<ApiResponse<RunnerOrderResponse>> {
    let order_id = parse_ulid(&order_id, "runner_order_id")?;
    let order = get_service(&state)?.admin_delivered(order_id).await?;
    Ok(ApiResponse::success(to_response(order)))
}

#[utoipa::path(
    post,
    path = "/admin/runner_orders/{id}/complete",
    params(("id" = String, Path, description = "Runner Order ID")),
    responses((status = 200, body = ApiResponse<RunnerOrderResponse>)),
    tag = "AdminRunnerOrder"
)]
pub async fn admin_complete_runner_order(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> AppResult<ApiResponse<RunnerOrderResponse>> {
    let order_id = parse_ulid(&order_id, "runner_order_id")?;
    let order = get_service(&state)?.admin_complete(order_id).await?;
    Ok(ApiResponse::success(to_response(order)))
}
