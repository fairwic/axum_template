//! Runner order handlers

use axum::{
    extract::{Path, Query, State},
    Json,
};
use axum_application::{CreateRunnerOrderInput, RunnerOrderService};
use axum_common_api::ApiResponse;
use axum_core_kernel::AppResult;
use axum_domain::order::entity::PayStatus;
use axum_domain::runner_order::entity::{RunnerOrder, RunnerOrderStatus};
use ulid::Ulid;

use crate::dtos::runner_order_dto::{
    CancelRunnerOrderRequest, CreateRunnerOrderRequest, ListRunnerOrdersQuery,
    PayRunnerOrderRequest, RunnerOrderResponse,
};
use crate::extractors::{parse_ulid, AuthUser};
use crate::state::AppState;

/// API DTO -> Application Input 映射
fn map_create_runner_order_input(
    payload: CreateRunnerOrderRequest,
    user_id: Ulid,
) -> AppResult<CreateRunnerOrderInput> {
    Ok(CreateRunnerOrderInput {
        user_id,
        store_id: parse_ulid(&payload.store_id, "store_id")?,
        express_company: payload.express_company,
        pickup_code: payload.pickup_code,
        delivery_address: payload.delivery_address,
        receiver_name: payload.receiver_name,
        receiver_phone: payload.receiver_phone,
        remark: payload.remark,
        distance_km: payload.distance_km,
    })
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
    Ok((**state.runner_order_service_ref()?).clone())
}

#[utoipa::path(
    post,
    path = "/runner_orders/create",
    request_body = CreateRunnerOrderRequest,
    responses((status = 200, body = ApiResponse<RunnerOrderResponse>)),
    tag = "RunnerOrder"
)]
/// 接口功能：create_runner_order，提交并创建跑腿订单
pub async fn create_runner_order(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateRunnerOrderRequest>,
) -> crate::error::ApiResult<ApiResponse<RunnerOrderResponse>> {
    let user_id = auth_user.user_id;
    let input = map_create_runner_order_input(payload, user_id)?;
    let order = get_service(&state)?.create(input).await?;

    Ok(ApiResponse::success(to_response(order)))
}

#[utoipa::path(
    post,
    path = "/runner_orders/pay",
    request_body = PayRunnerOrderRequest,
    responses((status = 200, body = ApiResponse<RunnerOrderResponse>)),
    tag = "RunnerOrder"
)]
/// 接口功能：pay_runner_order，发起跑腿订单支付
pub async fn pay_runner_order(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<PayRunnerOrderRequest>,
) -> crate::error::ApiResult<ApiResponse<RunnerOrderResponse>> {
    let user_id = auth_user.user_id;
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
/// 接口功能：list_runner_orders，查询当前用户跑腿订单列表
pub async fn list_runner_orders(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(query): Query<ListRunnerOrdersQuery>,
) -> crate::error::ApiResult<ApiResponse<Vec<RunnerOrderResponse>>> {
    let user_id = auth_user.user_id;
    let mut orders = get_service(&state)?.list_by_user(user_id).await?;
    if let Some(status) = query.status {
        orders.retain(|item| status_to_string(&item.status) == status);
    }
    Ok(ApiResponse::success(
        orders.into_iter().map(to_response).collect(),
    ))
}

#[utoipa::path(
    get,
    path = "/runner_orders/{id}",
    params(("id" = String, Path, description = "Runner Order ID")),
    responses((status = 200, body = ApiResponse<RunnerOrderResponse>)),
    tag = "RunnerOrder"
)]
/// 接口功能：get_runner_order，获取跑腿订单详情
pub async fn get_runner_order(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(runner_order_id): Path<String>,
) -> crate::error::ApiResult<ApiResponse<RunnerOrderResponse>> {
    let user_id = auth_user.user_id;
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
/// 接口功能：cancel_runner_order，取消跑腿订单
pub async fn cancel_runner_order(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(runner_order_id): Path<String>,
    Json(payload): Json<CancelRunnerOrderRequest>,
) -> crate::error::ApiResult<ApiResponse<RunnerOrderResponse>> {
    let user_id = auth_user.user_id;
    let runner_order_id = parse_ulid(&runner_order_id, "runner_order_id")?;
    let cancel_timeout_secs = state.biz_config.read().await.cancel_timeout_secs;
    let order = get_service(&state)?
        .cancel(
            user_id,
            runner_order_id,
            payload.reason,
            cancel_timeout_secs,
        )
        .await?;
    Ok(ApiResponse::success(to_response(order)))
}
