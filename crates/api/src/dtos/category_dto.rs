use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：CategoryQuery，商品分类查询参数
pub struct CategoryQuery {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
}

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：CategoryResponse，商品分类响应数据
pub struct CategoryResponse {
    /// 参数：id，记录唯一标识
    pub id: String,
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：name，名称
    pub name: String,
    /// 参数：sort_order，排序序号
    pub sort_order: i32,
    /// 参数：status，业务状态
    pub status: String,
}
