//! Admin store handlers

use axum::{
    extract::{Path, State},
    Json,
};
use axum_application::{CreateStoreInput, StoreService, UpdateStoreInput};
use axum_api_common::ApiResponse;
use axum_core_kernel::{AppError, AppResult};
use axum_domain::store::entity::{Store, StoreStatus};

use crate::dtos::store_dto::{
    AdminCreateStoreRequest, AdminUpdateStoreRequest, StoreAdminResponse,
};
use crate::extractors::parse_ulid;
use crate::state::AppState;

fn parse_store_status(value: &str) -> AppResult<StoreStatus> {
    match value {
        "OPEN" => Ok(StoreStatus::Open),
        "CLOSED" => Ok(StoreStatus::Closed),
        _ => Err(AppError::Validation("invalid store status".into())),
    }
}

fn map_create_store_input(payload: AdminCreateStoreRequest) -> AppResult<CreateStoreInput> {
    Ok(CreateStoreInput {
        name: payload.name,
        address: payload.address,
        lat: payload.lat,
        lng: payload.lng,
        phone: payload.phone,
        business_hours: payload.business_hours,
        status: parse_store_status(&payload.status)?,
        delivery_radius_km: payload.delivery_radius_km,
        delivery_fee_base: payload.delivery_fee_base,
        delivery_fee_per_km: payload.delivery_fee_per_km,
        runner_service_fee: payload.runner_service_fee,
    })
}

fn map_update_store_input(payload: AdminUpdateStoreRequest) -> AppResult<UpdateStoreInput> {
    Ok(UpdateStoreInput {
        name: payload.name,
        address: payload.address,
        lat: payload.lat,
        lng: payload.lng,
        phone: payload.phone,
        business_hours: payload.business_hours,
        status: parse_store_status(&payload.status)?,
        delivery_radius_km: payload.delivery_radius_km,
        delivery_fee_base: payload.delivery_fee_base,
        delivery_fee_per_km: payload.delivery_fee_per_km,
        runner_service_fee: payload.runner_service_fee,
    })
}

fn status_to_string(value: &StoreStatus) -> &'static str {
    match value {
        StoreStatus::Open => "OPEN",
        StoreStatus::Closed => "CLOSED",
    }
}

fn to_response(store: Store) -> StoreAdminResponse {
    StoreAdminResponse {
        id: store.id.to_string(),
        name: store.name,
        address: store.address,
        lat: store.lat,
        lng: store.lng,
        phone: store.phone,
        business_hours: store.business_hours,
        status: status_to_string(&store.status).to_string(),
        delivery_radius_km: store.delivery_radius_km,
        delivery_fee_base: store.delivery_fee_base,
        delivery_fee_per_km: store.delivery_fee_per_km,
        runner_service_fee: store.runner_service_fee,
    }
}

fn get_service(state: &AppState) -> StoreService {
    (*state.store_service).clone()
}

#[utoipa::path(
    get,
    path = "/admin/stores",
    responses((status = 200, body = ApiResponse<Vec<StoreAdminResponse>>)),
    tag = "AdminStore"
)]
/// 接口功能：admin_list_stores，后台查询门店列表
pub async fn admin_list_stores(
    State(state): State<AppState>,
) -> crate::error::ApiResult<ApiResponse<Vec<StoreAdminResponse>>> {
    let stores = get_service(&state).admin_list().await?;
    Ok(ApiResponse::success(
        stores.into_iter().map(to_response).collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/admin/stores",
    request_body = AdminCreateStoreRequest,
    responses((status = 200, body = ApiResponse<StoreAdminResponse>)),
    tag = "AdminStore"
)]
/// 接口功能：admin_create_store，后台创建门店
pub async fn admin_create_store(
    State(state): State<AppState>,
    Json(payload): Json<AdminCreateStoreRequest>,
) -> crate::error::ApiResult<ApiResponse<StoreAdminResponse>> {
    let input = map_create_store_input(payload)?;
    let store = get_service(&state).admin_create(input).await?;
    Ok(ApiResponse::success(to_response(store)))
}

#[utoipa::path(
    put,
    path = "/admin/stores/{id}",
    params(("id" = String, Path, description = "Store ID")),
    request_body = AdminUpdateStoreRequest,
    responses((status = 200, body = ApiResponse<StoreAdminResponse>)),
    tag = "AdminStore"
)]
/// 接口功能：admin_update_store，后台更新门店
pub async fn admin_update_store(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<AdminUpdateStoreRequest>,
) -> crate::error::ApiResult<ApiResponse<StoreAdminResponse>> {
    let store_id = parse_ulid(&id, "store_id")?;
    let input = map_update_store_input(payload)?;
    let store = get_service(&state).admin_update(store_id, input).await?;
    Ok(ApiResponse::success(to_response(store)))
}
