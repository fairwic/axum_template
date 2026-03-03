use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：OrderItemRequest，订单商品项请求参数
pub struct OrderItemRequest {
    /// 参数：product_id，商品唯一标识
    pub product_id: String,
    /// 参数：title_snapshot，商品标题快照
    pub title_snapshot: String,
    /// 参数：price_snapshot，下单时价格快照
    pub price_snapshot: i32,
    /// 参数：qty，商品数量
    pub qty: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：CreateOrderRequest，创建商品订单请求参数
pub struct CreateOrderRequest {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：delivery_type，配送方式
    pub delivery_type: String,
    /// 参数：items，明细项列表
    pub items: Vec<OrderItemRequest>,
    /// 参数：distance_km，距离（公里）
    pub distance_km: Option<f64>,
    /// 参数：address_id，收货地址唯一标识
    pub address_id: Option<String>,
    /// 参数：address_snapshot，收货地址快照
    pub address_snapshot: Option<serde_json::Value>,
    /// 参数：store_snapshot，门店信息快照
    pub store_snapshot: Option<serde_json::Value>,
    /// 参数：remark，备注信息
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：PreviewOrderRequest，订单预览请求参数
pub struct PreviewOrderRequest {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：delivery_type，配送方式
    pub delivery_type: String,
    /// 参数：items，明细项列表
    pub items: Vec<OrderItemRequest>,
    /// 参数：distance_km，距离（公里）
    pub distance_km: Option<f64>,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：PayOrderRequest，商品订单支付请求参数
pub struct PayOrderRequest {
    /// 参数：order_id，订单唯一标识
    pub order_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：CancelOrderRequest，取消商品订单请求参数
pub struct CancelOrderRequest {
    /// 参数：reason，取消原因
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：ListOrdersQuery，商品订单列表查询参数
pub struct ListOrdersQuery {
    /// 参数：status，业务状态
    pub status: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：OrderItemResponse，订单商品项响应数据
pub struct OrderItemResponse {
    /// 参数：product_id，商品唯一标识
    pub product_id: String,
    /// 参数：title_snapshot，商品标题快照
    pub title_snapshot: String,
    /// 参数：price_snapshot，下单时价格快照
    pub price_snapshot: i32,
    /// 参数：qty，商品数量
    pub qty: i32,
}

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：OrderResponse，订单详情响应数据
pub struct OrderResponse {
    /// 参数：order_id，订单唯一标识
    pub order_id: String,
    /// 参数：user_id，用户唯一标识
    pub user_id: String,
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：delivery_type，配送方式
    pub delivery_type: String,
    /// 参数：status，业务状态
    pub status: String,
    /// 参数：pay_status，支付状态
    pub pay_status: String,
    /// 参数：items，明细项列表
    pub items: Vec<OrderItemResponse>,
    /// 参数：amount_goods，商品总金额
    pub amount_goods: i32,
    /// 参数：amount_delivery_fee，配送费金额
    pub amount_delivery_fee: i32,
    /// 参数：amount_discount，优惠金额
    pub amount_discount: i32,
    /// 参数：amount_payable，应付金额
    pub amount_payable: i32,
    /// 参数：distance_km，距离（公里）
    pub distance_km: Option<f64>,
    /// 参数：address_snapshot，收货地址快照
    pub address_snapshot: Option<serde_json::Value>,
    /// 参数：store_snapshot，门店信息快照
    pub store_snapshot: Option<serde_json::Value>,
    /// 参数：remark，备注信息
    pub remark: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：OrderPreviewResponse，订单预览响应数据
pub struct OrderPreviewResponse {
    /// 参数：amount_goods，商品总金额
    pub amount_goods: i32,
    /// 参数：amount_delivery_fee，配送费金额
    pub amount_delivery_fee: i32,
    /// 参数：amount_discount，优惠金额
    pub amount_discount: i32,
    /// 参数：amount_payable，应付金额
    pub amount_payable: i32,
    /// 参数：distance_km，距离（公里）
    pub distance_km: Option<f64>,
    /// 参数：deliverable，是否可配送
    pub deliverable: bool,
}
