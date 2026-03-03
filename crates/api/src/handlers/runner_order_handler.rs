//! Runner order handlers

use axum::{extract::{Path, Query, State}, http::HeaderMap, Json};
use axum_application::{CreateRunnerOrderInput, RunnerOrderService};
use axum_common::{ApiResponse, AppError, AppResult};
use axum_domain::order::entity::PayStatus;
use axum_domain::runner_order::entity::{RunnerOrder, RunnerOrderStatus};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use ulid::Ulid;

use crate::state::AppState;

const USER_ID_HEADER: &str = "x-user-id";

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateRunnerOrderRequest {
    pub store_id: String,
    pub express_company: String,
    pub pickup_code: String,
    pub delivery_address: String,
    pub receiver_name: String,
    pub receiver_phone: String,
    pub remark: Option<String>,
    pub distance_km: Option<f64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PayRunnerOrderRequest {
    pub runner_order_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CancelRunnerOrderRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListRunnerOrdersQuery {
    pub status: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RunnerOrderResponse {
    pub runner_order_id: String,
    pub user_id: String,
    pub store_id: String,
    pub status: String,
    pub pay_status: String,
    pub express_company: String,
    pub pickup_code: String,
    pub delivery_address: String,
    pub receiver_name: String,
    pub receiver_phone: String,
    pub remark: Option<String>,
    pub service_fee: i32,
    pub distance_km: Option<f64>,
    pub amount_payable: i32,
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

fn status_to_string(value: &RunnerOrderStatus) -> &'static str {
    match value {
        RunnerOrderStatus::PendingPay => "PENDING_PAY",
        RunnerOrderStatus::PendingAccept => "PENDING_ACCEPT",
        RunnerOrderStatus::Processing => "PROCESSING",
        RunnerOrderStatus::Delivered => "DELIVERED",
        RunnerOrderStatus::Completed => "COMPLETED",
        RunnerOrderStatus::Canceled => "CANCELED",
        RunnerOrderStatus::Closed => "CLOSED",
    }
}

fn pay_status_to_string(value: &PayStatus) -> &'static str {
    match value {
        PayStatus::Unpaid => "UNPAID",
        PayStatus::Paid => "PAID",
        PayStatus::Refunded => "REFUNDED",
    }
}

fn to_response(order: RunnerOrder) -> RunnerOrderResponse {
    RunnerOrderResponse {
        runner_order_id: order.id.to_string(),
        user_id: order.user_id.to_string(),
        store_id: order.store_id.to_string(),
        status: status_to_string(&order.status).to_string(),
        pay_status: pay_status_to_string(&order.pay_status).to_string(),
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

fn get_service(state: &AppState) -> AppResult<RunnerOrderService> {
    state
        .runner_order_service
        .as_ref()
        .cloned()
        .map(|item| (*item).clone())
        .ok_or_else(|| AppError::Internal("runner order service not initialized".into()))
}

#[utoipa::path(
    post,
    path = "/runner_orders/create",
    request_body = CreateRunnerOrderRequest,
    responses((status = 200, body = ApiResponse<RunnerOrderResponse>)),
    tag = "RunnerOrder"
)]
pub async fn create_runner_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateRunnerOrderRequest>,
) -> AppResult<ApiResponse<RunnerOrderResponse>> {
    let user_id = parse_user_id(&headers)?;
    let store_id = parse_ulid(&payload.store_id, "store_id")?;

    let order = get_service(&state)?
        .create(CreateRunnerOrderInput {
            user_id,
            store_id,
            express_company: payload.express_company,
            pickup_code: payload.pickup_code,
            delivery_address: payload.delivery_address,
            receiver_name: payload.receiver_name,
            receiver_phone: payload.receiver_phone,
            remark: payload.remark,
            distance_km: payload.distance_km,
        })
        .await?;

    Ok(ApiResponse::success(to_response(order)))
}

#[utoipa::path(
    post,
    path = "/runner_orders/pay",
    request_body = PayRunnerOrderRequest,
    responses((status = 200, body = ApiResponse<RunnerOrderResponse>)),
    tag = "RunnerOrder"
)]
pub async fn pay_runner_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<PayRunnerOrderRequest>,
) -> AppResult<ApiResponse<RunnerOrderResponse>> {
    let user_id = parse_user_id(&headers)?;
    let runner_order_id = parse_ulid(&payload.runner_order_id, "runner_order_id")?;
    let order = get_service(&state)?.pay(user_id, runner_order_id).await?;
    Ok(ApiResponse::success(to_response(order)))
}

#[utoipa::path(
    get,
    path = "/runner_orders",
    params(("status" = Option<String>, Query, description = "Status filter")),
    responses((status = 200, body = ApiResponse<Vec<RunnerOrderResponse>>)),
    tag = "RunnerOrder"
)]
pub async fn list_runner_orders(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListRunnerOrdersQuery>,
) -> AppResult<ApiResponse<Vec<RunnerOrderResponse>>> {
    let user_id = parse_user_id(&headers)?;
    let mut orders = get_service(&state)?.list_by_user(user_id).await?;
    if let Some(status) = query.status {
        orders.retain(|item| status_to_string(&item.status) == status);
    }
    Ok(ApiResponse::success(orders.into_iter().map(to_response).collect()))
}

#[utoipa::path(
    get,
    path = "/runner_orders/{id}",
    params(("id" = String, Path, description = "Runner Order ID")),
    responses((status = 200, body = ApiResponse<RunnerOrderResponse>)),
    tag = "RunnerOrder"
)]
pub async fn get_runner_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(runner_order_id): Path<String>,
) -> AppResult<ApiResponse<RunnerOrderResponse>> {
    let user_id = parse_user_id(&headers)?;
    let runner_order_id = parse_ulid(&runner_order_id, "runner_order_id")?;
    let order = get_service(&state)?
        .get_by_user(user_id, runner_order_id)
        .await?;
    Ok(ApiResponse::success(to_response(order)))
}

#[utoipa::path(
    post,
    path = "/runner_orders/{id}/cancel",
    params(("id" = String, Path, description = "Runner Order ID")),
    request_body = CancelRunnerOrderRequest,
    responses((status = 200, body = ApiResponse<RunnerOrderResponse>)),
    tag = "RunnerOrder"
)]
pub async fn cancel_runner_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(runner_order_id): Path<String>,
    Json(payload): Json<CancelRunnerOrderRequest>,
) -> AppResult<ApiResponse<RunnerOrderResponse>> {
    let user_id = parse_user_id(&headers)?;
    let runner_order_id = parse_ulid(&runner_order_id, "runner_order_id")?;
    let order = get_service(&state)?
        .cancel(user_id, runner_order_id, payload.reason)
        .await?;
    Ok(ApiResponse::success(to_response(order)))
}
