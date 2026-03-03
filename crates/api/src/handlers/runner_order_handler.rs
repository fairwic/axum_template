//! Runner order handlers

use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use axum_application::{CreateRunnerOrderInput, RunnerOrderService};
use axum_common::{ApiResponse, AppError, AppResult};
use axum_domain::order::entity::PayStatus;
use axum_domain::runner_order::entity::{RunnerOrder, RunnerOrderStatus};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use utoipa::ToSchema;

use crate::state::AppState;

const USER_ID_HEADER: &str = "x-user-id";

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：CreateRunnerOrderRequest，创建跑腿订单请求参数
pub struct CreateRunnerOrderRequest {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：express_company，快递公司名称
    pub express_company: String,
    /// 参数：pickup_code，取件码
    pub pickup_code: String,
    /// 参数：delivery_address，送达地址文本
    pub delivery_address: String,
    /// 参数：receiver_name，收件人姓名
    pub receiver_name: String,
    /// 参数：receiver_phone，收件人手机号
    pub receiver_phone: String,
    /// 参数：remark，备注信息
    pub remark: Option<String>,
    /// 参数：distance_km，距离（公里）
    pub distance_km: Option<f64>,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：PayRunnerOrderRequest，跑腿订单支付请求参数
pub struct PayRunnerOrderRequest {
    /// 参数：runner_order_id，跑腿订单唯一标识
    pub runner_order_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：CancelRunnerOrderRequest，取消跑腿订单请求参数
pub struct CancelRunnerOrderRequest {
    /// 参数：reason，取消原因
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：ListRunnerOrdersQuery，跑腿订单列表查询参数
pub struct ListRunnerOrdersQuery {
    /// 参数：status，业务状态
    pub status: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：RunnerOrderResponse，跑腿订单响应数据
pub struct RunnerOrderResponse {
    /// 参数：runner_order_id，跑腿订单唯一标识
    pub runner_order_id: String,
    /// 参数：user_id，用户唯一标识
    pub user_id: String,
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：status，业务状态
    pub status: String,
    /// 参数：pay_status，支付状态
    pub pay_status: String,
    /// 参数：express_company，快递公司名称
    pub express_company: String,
    /// 参数：pickup_code，取件码
    pub pickup_code: String,
    /// 参数：delivery_address，送达地址文本
    pub delivery_address: String,
    /// 参数：receiver_name，收件人姓名
    pub receiver_name: String,
    /// 参数：receiver_phone，收件人手机号
    pub receiver_phone: String,
    /// 参数：remark，备注信息
    pub remark: Option<String>,
    /// 参数：service_fee，跑腿服务费
    pub service_fee: i32,
    /// 参数：distance_km，距离（公里）
    pub distance_km: Option<f64>,
    /// 参数：amount_payable，应付金额
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
/// 接口功能：create_runner_order，提交并创建跑腿订单
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
/// 接口功能：pay_runner_order，发起跑腿订单支付
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
/// 接口功能：list_runner_orders，查询当前用户跑腿订单列表
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
/// 接口功能：cancel_runner_order，取消跑腿订单
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
