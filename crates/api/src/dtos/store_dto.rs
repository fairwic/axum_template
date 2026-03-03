use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
