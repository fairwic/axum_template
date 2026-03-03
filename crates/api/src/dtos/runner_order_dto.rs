use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：CreateRunnerOrderRequest，创建跑腿订单请求参数
pub struct CreateRunnerOrderRequest {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：express_company，快递公司名称
    pub express_company: String,
    /// 参数：pickup_code，取件码
    pub pickup_code: String,
    /// 参数：delivery_address，送达地址文本
    pub delivery_address: String,
    /// 参数：receiver_name，收件人姓名
    pub receiver_name: String,
    /// 参数：receiver_phone，收件人手机号
    pub receiver_phone: String,
    /// 参数：remark，备注信息
    pub remark: Option<String>,
    /// 参数：distance_km，距离（公里）
    pub distance_km: Option<f64>,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：PayRunnerOrderRequest，跑腿订单支付请求参数
pub struct PayRunnerOrderRequest {
    /// 参数：runner_order_id，跑腿订单唯一标识
    pub runner_order_id: String,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：CancelRunnerOrderRequest，取消跑腿订单请求参数
pub struct CancelRunnerOrderRequest {
    /// 参数：reason，取消原因
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：ListRunnerOrdersQuery，跑腿订单列表查询参数
pub struct ListRunnerOrdersQuery {
    /// 参数：status，业务状态
    pub status: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：RunnerOrderResponse，跑腿订单响应数据
pub struct RunnerOrderResponse {
    /// 参数：runner_order_id，跑腿订单唯一标识
    pub runner_order_id: String,
    /// 参数：user_id，用户唯一标识
    pub user_id: String,
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：status，业务状态
    pub status: String,
    /// 参数：pay_status，支付状态
    pub pay_status: String,
    /// 参数：express_company，快递公司名称
    pub express_company: String,
    /// 参数：pickup_code，取件码
    pub pickup_code: String,
    /// 参数：delivery_address，送达地址文本
    pub delivery_address: String,
    /// 参数：receiver_name，收件人姓名
    pub receiver_name: String,
    /// 参数：receiver_phone，收件人手机号
    pub receiver_phone: String,
    /// 参数：remark，备注信息
    pub remark: Option<String>,
    /// 参数：service_fee，跑腿服务费
    pub service_fee: i32,
    /// 参数：distance_km，距离（公里）
    pub distance_km: Option<f64>,
    /// 参数：amount_payable，应付金额
    pub amount_payable: i32,
}
