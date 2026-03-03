use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：CartQuery，购物车查询参数
pub struct CartQuery {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：AddItemRequest，添加购物车商品请求参数
pub struct AddItemRequest {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：product_id，商品唯一标识
    pub product_id: String,
    /// 参数：qty，商品数量
    pub qty: i32,
    /// 参数：price_snapshot，下单时价格快照
    pub price_snapshot: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：UpdateQtyRequest，更新购物车数量请求参数
pub struct UpdateQtyRequest {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：product_id，商品唯一标识
    pub product_id: String,
    /// 参数：qty，商品数量
    pub qty: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：RemoveItemRequest，移除购物车商品请求参数
pub struct RemoveItemRequest {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：product_id，商品唯一标识
    pub product_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：ClearCartRequest，清空购物车请求参数
pub struct ClearCartRequest {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：CartItemResponse，购物车商品项响应数据
pub struct CartItemResponse {
    /// 参数：product_id，商品唯一标识
    pub product_id: String,
    /// 参数：qty，商品数量
    pub qty: i32,
    /// 参数：price_snapshot，下单时价格快照
    pub price_snapshot: i32,
}

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：CartResponse，购物车响应数据
pub struct CartResponse {
    /// 参数：cart_id，购物车唯一标识
    pub cart_id: String,
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：items，明细项列表
    pub items: Vec<CartItemResponse>,
}
