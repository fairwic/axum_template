use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;
use validator::Validate;

use axum_application::{CreateAddressInput, UpdateAddressInput};
use axum_domain::Address;

#[derive(Debug, Deserialize, ToSchema, Validate)]
#[schema(example = json!({
    "name": "张三",
    "phone": "13800138000",
    "detail": "北京市朝阳区望京街道xx路xx号",
    "lat": 39.9042,
    "lng": 116.4074,
    "is_default": true
}))]
/// 创建收货地址请求
pub struct CreateAddressRequest {
    /// 收货人姓名（1-50 字符）
    #[validate(length(min = 1, max = 50, message = "收货人姓名长度必须在 1-50 字符之间"))]
    #[schema(example = "张三")]
    pub name: String,

    /// 手机号（11 位数字）
    #[validate(
        length(equal = 11, message = "手机号必须为 11 位数字"),
        regex(path = "PHONE_REGEX", message = "手机号格式不正确")
    )]
    #[schema(example = "13800138000")]
    pub phone: String,

    /// 详细地址（1-200 字符）
    #[validate(length(min = 1, max = 200, message = "详细地址长度必须在 1-200 字符之间"))]
    #[schema(example = "北京市朝阳区望京街道xx路xx号")]
    pub detail: String,

    /// 纬度（-90.0 至 90.0，可选）
    #[validate(range(min = -90.0, max = 90.0, message = "纬度必须在 -90.0 至 90.0 之间"))]
    #[schema(example = 39.9042)]
    pub lat: Option<f64>,

    /// 经度（-180.0 至 180.0，可选）
    #[validate(range(min = -180.0, max = 180.0, message = "经度必须在 -180.0 至 180.0 之间"))]
    #[schema(example = 116.4074)]
    pub lng: Option<f64>,

    /// 是否设为默认地址
    #[schema(example = true)]
    pub is_default: bool,
}

#[derive(Debug, Deserialize, ToSchema, Validate)]
#[schema(example = json!({
    "name": "李四",
    "phone": "13900139000",
    "detail": "上海市浦东新区陆家嘴xx路xx号",
    "lat": 31.2304,
    "lng": 121.4737,
    "is_default": false
}))]
/// 更新收货地址请求
pub struct UpdateAddressRequest {
    /// 收货人姓名（1-50 字符）
    #[validate(length(min = 1, max = 50, message = "收货人姓名长度必须在 1-50 字符之间"))]
    #[schema(example = "李四")]
    pub name: String,

    /// 手机号（11 位数字）
    #[validate(
        length(equal = 11, message = "手机号必须为 11 位数字"),
        regex(path = "PHONE_REGEX", message = "手机号格式不正确")
    )]
    #[schema(example = "13900139000")]
    pub phone: String,

    /// 详细地址（1-200 字符）
    #[validate(length(min = 1, max = 200, message = "详细地址长度必须在 1-200 字符之间"))]
    #[schema(example = "上海市浦东新区陆家嘴xx路xx号")]
    pub detail: String,

    /// 纬度（-90.0 至 90.0，可选）
    #[validate(range(min = -90.0, max = 90.0, message = "纬度必须在 -90.0 至 90.0 之间"))]
    #[schema(example = 31.2304)]
    pub lat: Option<f64>,

    /// 经度（-180.0 至 180.0，可选）
    #[validate(range(min = -180.0, max = 180.0, message = "经度必须在 -180.0 至 180.0 之间"))]
    #[schema(example = 121.4737)]
    pub lng: Option<f64>,

    /// 是否设为默认地址
    #[schema(example = false)]
    pub is_default: bool,
}

#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "address_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "name": "张三",
    "phone": "13800138000",
    "detail": "北京市朝阳区望京街道xx路xx号",
    "lat": 39.9042,
    "lng": 116.4074,
    "is_default": true
}))]
/// 收货地址响应
pub struct AddressResponse {
    /// 收货地址唯一标识（ULID 格式）
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub address_id: String,

    /// 收货人姓名
    #[schema(example = "张三")]
    pub name: String,

    /// 手机号
    #[schema(example = "13800138000")]
    pub phone: String,

    /// 详细地址
    #[schema(example = "北京市朝阳区望京街道xx路xx号")]
    pub detail: String,

    /// 纬度
    #[schema(example = 39.9042)]
    pub lat: Option<f64>,

    /// 经度
    #[schema(example = 116.4074)]
    pub lng: Option<f64>,

    /// 是否默认地址
    #[schema(example = true)]
    pub is_default: bool,
}

// ========== 转换实现 ==========

/// Request → Application Input 转换
impl From<CreateAddressRequest> for CreateAddressInput {
    fn from(req: CreateAddressRequest) -> Self {
        Self {
            name: req.name,
            phone: req.phone,
            detail: req.detail,
            lat: req.lat,
            lng: req.lng,
            is_default: req.is_default,
        }
    }
}

impl From<UpdateAddressRequest> for UpdateAddressInput {
    fn from(req: UpdateAddressRequest) -> Self {
        Self {
            name: req.name,
            phone: req.phone,
            detail: req.detail,
            lat: req.lat,
            lng: req.lng,
            is_default: req.is_default,
        }
    }
}

/// Domain Entity → Response 转换
impl From<Address> for AddressResponse {
    fn from(addr: Address) -> Self {
        Self {
            address_id: addr.id.to_string(),
            name: addr.name,
            phone: addr.phone,
            detail: addr.detail,
            lat: addr.lat,
            lng: addr.lng,
            is_default: addr.is_default,
        }
    }
}

// ========== 验证正则 ==========

lazy_static::lazy_static! {
    /// 中国大陆手机号正则（1 开头 11 位数字）
    static ref PHONE_REGEX: regex::Regex = regex::Regex::new(r"^1[3-9]\d{9}$").unwrap();
}
