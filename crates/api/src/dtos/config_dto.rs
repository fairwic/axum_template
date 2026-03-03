use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：ConfigResponse，全局配置响应数据
pub struct ConfigResponse {
    /// 参数：delivery_free_radius_km，免配送半径（公里）
    pub delivery_free_radius_km: f64,
    /// 参数：runner_service_fee，跑腿服务费（分）
    pub runner_service_fee: i32,
    /// 参数：customer_service_phone，客服电话
    pub customer_service_phone: String,
    /// 参数：runner_banner_enabled，跑腿 Banner 开关
    pub runner_banner_enabled: bool,
    /// 参数：runner_banner_text，跑腿 Banner 文案
    pub runner_banner_text: String,
    /// 参数：pay_timeout_secs，未支付超时关单秒数
    pub pay_timeout_secs: u64,
    /// 参数：auto_accept_secs，待接单自动接单秒数
    pub auto_accept_secs: u64,
    /// 参数：cancel_timeout_secs，可取消时间窗秒数
    pub cancel_timeout_secs: u64,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：UpdateConfigRequest，后台更新全局配置请求参数
pub struct UpdateConfigRequest {
    /// 参数：delivery_free_radius_km，免配送半径（公里）
    pub delivery_free_radius_km: f64,
    /// 参数：runner_service_fee，跑腿服务费（分）
    pub runner_service_fee: i32,
    /// 参数：customer_service_phone，客服电话
    pub customer_service_phone: String,
    /// 参数：runner_banner_enabled，跑腿 Banner 开关
    pub runner_banner_enabled: bool,
    /// 参数：runner_banner_text，跑腿 Banner 文案
    pub runner_banner_text: String,
    /// 参数：pay_timeout_secs，未支付超时关单秒数
    pub pay_timeout_secs: u64,
    /// 参数：auto_accept_secs，待接单自动接单秒数
    pub auto_accept_secs: u64,
    /// 参数：cancel_timeout_secs，可取消时间窗秒数
    pub cancel_timeout_secs: u64,
}
