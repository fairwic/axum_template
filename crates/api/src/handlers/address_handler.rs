//! Address handlers

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use axum_application::{AddressService, CreateAddressInput, UpdateAddressInput};
use axum_common::{ApiResponse, AppError, AppResult};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use utoipa::ToSchema;

use crate::state::AppState;

const USER_ID_HEADER: &str = "x-user-id";

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：CreateAddressRequest，新增收货地址请求参数
pub struct CreateAddressRequest {
    /// 参数：name，名称
    pub name: String,
    /// 参数：phone，手机号
    pub phone: String,
    /// 参数：detail，收货地址详细信息
    pub detail: String,
    /// 参数：lat，纬度坐标
    pub lat: Option<f64>,
    /// 参数：lng，经度坐标
    pub lng: Option<f64>,
    /// 参数：is_default，是否默认地址
    pub is_default: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：UpdateAddressRequest，更新收货地址请求参数
pub struct UpdateAddressRequest {
    /// 参数：name，名称
    pub name: String,
    /// 参数：phone，手机号
    pub phone: String,
    /// 参数：detail，收货地址详细信息
    pub detail: String,
    /// 参数：lat，纬度坐标
    pub lat: Option<f64>,
    /// 参数：lng，经度坐标
    pub lng: Option<f64>,
    /// 参数：is_default，是否默认地址
    pub is_default: bool,
}

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：AddressResponse，收货地址响应数据
pub struct AddressResponse {
    /// 参数：address_id，收货地址唯一标识
    pub address_id: String,
    /// 参数：name，名称
    pub name: String,
    /// 参数：phone，手机号
    pub phone: String,
    /// 参数：detail，收货地址详细信息
    pub detail: String,
    /// 参数：lat，纬度坐标
    pub lat: Option<f64>,
    /// 参数：lng，经度坐标
    pub lng: Option<f64>,
    /// 参数：is_default，是否默认地址
    pub is_default: bool,
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

fn to_response(address: axum_domain::Address) -> AddressResponse {
    AddressResponse {
        address_id: address.id.to_string(),
        name: address.name,
        phone: address.phone,
        detail: address.detail,
        lat: address.lat,
        lng: address.lng,
        is_default: address.is_default,
    }
}

fn get_service(state: &AppState) -> AppResult<AddressService> {
    state
        .address_service
        .as_ref()
        .cloned()
        .map(|item| (*item).clone())
        .ok_or_else(|| AppError::Internal("address service not initialized".into()))
}

#[utoipa::path(
    get,
    path = "/addresses",
    responses((status = 200, body = ApiResponse<Vec<AddressResponse>>)),
    tag = "Address"
)]
/// 接口功能：list_addresses，查询当前用户收货地址列表
pub async fn list_addresses(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<ApiResponse<Vec<AddressResponse>>> {
    let user_id = parse_user_id(&headers)?;
    let addresses = get_service(&state)?.list(user_id).await?;
    Ok(ApiResponse::success(
        addresses.into_iter().map(to_response).collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/addresses",
    request_body = CreateAddressRequest,
    responses((status = 200, body = ApiResponse<AddressResponse>)),
    tag = "Address"
)]
/// 接口功能：create_address，创建用户收货地址
pub async fn create_address(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateAddressRequest>,
) -> AppResult<ApiResponse<AddressResponse>> {
    let user_id = parse_user_id(&headers)?;
    let address = get_service(&state)?
        .create(
            user_id,
            CreateAddressInput {
                name: payload.name,
                phone: payload.phone,
                detail: payload.detail,
                lat: payload.lat,
                lng: payload.lng,
                is_default: payload.is_default,
            },
        )
        .await?;
    Ok(ApiResponse::success(to_response(address)))
}

#[utoipa::path(
    put,
    path = "/addresses/{id}",
    params(("id" = String, Path, description = "Address ID")),
    request_body = UpdateAddressRequest,
    responses((status = 200, body = ApiResponse<AddressResponse>)),
    tag = "Address"
)]
/// 接口功能：update_address，更新收货地址信息
pub async fn update_address(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(address_id): Path<String>,
    Json(payload): Json<UpdateAddressRequest>,
) -> AppResult<ApiResponse<AddressResponse>> {
    let user_id = parse_user_id(&headers)?;
    let address_id = parse_ulid(&address_id, "address_id")?;
    let address = get_service(&state)?
        .update(
            user_id,
            address_id,
            UpdateAddressInput {
                name: payload.name,
                phone: payload.phone,
                detail: payload.detail,
                lat: payload.lat,
                lng: payload.lng,
                is_default: payload.is_default,
            },
        )
        .await?;
    Ok(ApiResponse::success(to_response(address)))
}

#[utoipa::path(
    delete,
    path = "/addresses/{id}",
    params(("id" = String, Path, description = "Address ID")),
    responses((status = 200, body = ApiResponse<bool>)),
    tag = "Address"
)]
/// 接口功能：delete_address，删除指定收货地址
pub async fn delete_address(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(address_id): Path<String>,
) -> AppResult<ApiResponse<bool>> {
    let user_id = parse_user_id(&headers)?;
    let address_id = parse_ulid(&address_id, "address_id")?;
    get_service(&state)?.delete(user_id, address_id).await?;
    Ok(ApiResponse::success(true))
}

#[utoipa::path(
    post,
    path = "/addresses/{id}/set_default",
    params(("id" = String, Path, description = "Address ID")),
    responses((status = 200, body = ApiResponse<AddressResponse>)),
    tag = "Address"
)]
/// 接口功能：set_default_address，设置默认收货地址
pub async fn set_default_address(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(address_id): Path<String>,
) -> AppResult<ApiResponse<AddressResponse>> {
    let user_id = parse_user_id(&headers)?;
    let address_id = parse_ulid(&address_id, "address_id")?;
    let address = get_service(&state)?
        .set_default(user_id, address_id)
        .await?;
    Ok(ApiResponse::success(to_response(address)))
}
