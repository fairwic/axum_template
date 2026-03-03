use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
