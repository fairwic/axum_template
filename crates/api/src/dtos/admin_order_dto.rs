use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：AdminListOrdersQuery，后台订单列表查询参数
pub struct AdminListOrdersQuery {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：status，业务状态
    pub status: Option<String>,
}
