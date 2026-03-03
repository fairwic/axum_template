//! Store handlers

use axum::{
    extract::{Query, State},
    http::HeaderMap,
    Json,
};
use axum_common::{ApiResponse, AppError, AppResult};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use utoipa::ToSchema;

use axum_domain::store::entity::{Store, StoreStatus};

use crate::state::AppState;

const USER_ID_HEADER: &str = "x-user-id";

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：NearbyQuery，附近门店查询参数
pub struct NearbyQuery {
    /// 参数：lat，纬度坐标
    pub lat: f64,
    /// 参数：lng，经度坐标
    pub lng: f64,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：SelectStoreRequest，选择门店请求参数
pub struct SelectStoreRequest {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：StoreNearbyResponse，附近门店响应数据
pub struct StoreNearbyResponse {
    /// 参数：id，记录唯一标识
    pub id: String,
    /// 参数：name，名称
    pub name: String,
    /// 参数：address，地址信息
    pub address: String,
    /// 参数：lat，纬度坐标
    pub lat: f64,
    /// 参数：lng，经度坐标
    pub lng: f64,
    /// 参数：phone，手机号
    pub phone: String,
    /// 参数：business_hours，营业时间
    pub business_hours: String,
    /// 参数：status，业务状态
    pub status: String,
    /// 参数：distance_km，距离（公里）
    pub distance_km: f64,
    /// 参数：deliverable，是否可配送
    pub deliverable: bool,
    /// 参数：delivery_fee，配送费
    pub delivery_fee: i32,
}

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：StoreResponse，门店响应数据
pub struct StoreResponse {
    /// 参数：id，记录唯一标识
    pub id: String,
    /// 参数：name，名称
    pub name: String,
    /// 参数：address，地址信息
    pub address: String,
    /// 参数：lat，纬度坐标
    pub lat: f64,
    /// 参数：lng，经度坐标
    pub lng: f64,
    /// 参数：phone，手机号
    pub phone: String,
    /// 参数：business_hours，营业时间
    pub business_hours: String,
    /// 参数：status，业务状态
    pub status: String,
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

fn status_to_string(status: &StoreStatus) -> String {
    match status {
        StoreStatus::Open => "OPEN".to_string(),
        StoreStatus::Closed => "CLOSED".to_string(),
    }
}

fn to_response(
    store: Store,
    distance_km: f64,
    deliverable: bool,
    delivery_fee: i32,
) -> StoreNearbyResponse {
    StoreNearbyResponse {
        id: store.id.to_string(),
        name: store.name,
        address: store.address,
        lat: store.lat,
        lng: store.lng,
        phone: store.phone,
        business_hours: store.business_hours,
        status: status_to_string(&store.status),
        distance_km,
        deliverable,
        delivery_fee,
    }
}

fn to_store_response(store: Store) -> StoreResponse {
    StoreResponse {
        id: store.id.to_string(),
        name: store.name,
        address: store.address,
        lat: store.lat,
        lng: store.lng,
        phone: store.phone,
        business_hours: store.business_hours,
        status: status_to_string(&store.status),
    }
}

#[utoipa::path(
    get,
    path = "/stores/nearby",
    params(("lat" = f64, Query, description = "纬度"), ("lng" = f64, Query, description = "经度")),
    responses((status = 200, description = "Nearby stores", body = ApiResponse<Vec<StoreNearbyResponse>>)),
    tag = "Store"
)]
/// 接口功能：nearby_stores，查询附近可服务门店
pub async fn nearby_stores(
    State(state): State<AppState>,
    Query(query): Query<NearbyQuery>,
) -> AppResult<ApiResponse<Vec<StoreNearbyResponse>>> {
    let items = state.store_service.nearby(query.lat, query.lng).await?;
    let data = items
        .into_iter()
        .map(|item| {
            to_response(
                item.store,
                item.distance_km,
                item.deliverable,
                item.delivery_fee,
            )
        })
        .collect();
    Ok(ApiResponse::success(data))
}

#[utoipa::path(
    post,
    path = "/stores/select",
    request_body = SelectStoreRequest,
    responses((status = 200, description = "Select current store", body = ApiResponse<StoreResponse>)),
    tag = "Store"
)]
/// 接口功能：select_store，选择并保存当前门店
pub async fn select_store(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SelectStoreRequest>,
) -> AppResult<ApiResponse<StoreResponse>> {
    let user_id = parse_user_id(&headers)?;
    let store_id = parse_ulid(&payload.store_id, "store_id")?;

    state.user_service.get_by_id(user_id).await?;
    let store = state.store_service.get_by_id(store_id).await?;
    state
        .user_service
        .set_current_store(user_id, store_id)
        .await?;

    Ok(ApiResponse::success(to_store_response(store)))
}

#[utoipa::path(
    get,
    path = "/stores/current",
    responses((status = 200, description = "Current selected store", body = ApiResponse<StoreResponse>)),
    tag = "Store"
)]
/// 接口功能：current_store，获取用户当前已选择门店
pub async fn current_store(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<ApiResponse<StoreResponse>> {
    let user_id = parse_user_id(&headers)?;
    let user = state.user_service.get_by_id(user_id).await?;
    let store_id = user
        .current_store_id
        .ok_or_else(|| AppError::NotFound("current store not selected".into()))?;
    let store = state.store_service.get_by_id(store_id).await?;

    Ok(ApiResponse::success(to_store_response(store)))
}
