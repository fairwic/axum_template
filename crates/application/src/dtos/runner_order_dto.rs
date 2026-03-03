//! Runner order application DTOs

use ulid::Ulid;

/// 应用层输入：创建跑腿订单
#[derive(Debug, Clone)]
pub struct CreateRunnerOrderInput {
    pub user_id: Ulid,
    pub store_id: Ulid,
    pub express_company: String,
    pub pickup_code: String,
    pub delivery_address: String,
    pub receiver_name: String,
    pub receiver_phone: String,
    pub remark: Option<String>,
    pub distance_km: Option<f64>,
}
