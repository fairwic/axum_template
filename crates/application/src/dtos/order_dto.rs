//! Order application DTOs

use axum_domain::order::entity::{DeliveryType, GoodsOrderItem};
use ulid::Ulid;

/// 应用层输入：创建商品订单
#[derive(Debug, Clone)]
pub struct CreateGoodsOrderInput {
    pub user_id: Ulid,
    pub store_id: Ulid,
    pub delivery_type: DeliveryType,
    pub items: Vec<GoodsOrderItem>,
    pub distance_km: Option<f64>,
    pub address_snapshot: Option<serde_json::Value>,
    pub store_snapshot: Option<serde_json::Value>,
    pub remark: Option<String>,
}

/// 应用层输出：订单预览
#[derive(Debug, Clone)]
pub struct OrderPreview {
    pub amount_goods: i32,
    pub amount_delivery_fee: i32,
    pub amount_discount: i32,
    pub amount_payable: i32,
    pub distance_km: Option<f64>,
    pub deliverable: bool,
}
