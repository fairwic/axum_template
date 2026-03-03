use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：ProductListQuery，商品列表查询参数
pub struct ProductListQuery {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：category_id，分类唯一标识
    pub category_id: String,
    #[serde(default = "default_page")]
    /// 参数：page，页码
    pub page: i64,
    #[serde(default = "default_page_size")]
    /// 参数：page_size，每页条数
    pub page_size: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：ProductSearchQuery，商品搜索查询参数
pub struct ProductSearchQuery {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：keyword，搜索关键词
    pub keyword: String,
    #[serde(default = "default_page")]
    /// 参数：page，页码
    pub page: i64,
    #[serde(default = "default_page_size")]
    /// 参数：page_size，每页条数
    pub page_size: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
/// DTO定义：ProductDetailQuery，商品详情查询参数
pub struct ProductDetailQuery {
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    20
}

#[derive(Debug, Serialize, ToSchema)]
/// DTO定义：ProductResponse，商品响应数据
pub struct ProductResponse {
    /// 参数：id，记录唯一标识
    pub id: String,
    /// 参数：store_id，门店唯一标识
    pub store_id: String,
    /// 参数：category_id，分类唯一标识
    pub category_id: String,
    /// 参数：title，商品标题
    pub title: String,
    /// 参数：subtitle，商品副标题
    pub subtitle: Option<String>,
    /// 参数：cover_image，封面图 URL
    pub cover_image: String,
    /// 参数：images，商品图片列表
    pub images: Vec<String>,
    /// 参数：price，当前售价
    pub price: i32,
    /// 参数：original_price，商品原价
    pub original_price: Option<i32>,
    /// 参数：stock，库存数量
    pub stock: i32,
    /// 参数：status，业务状态
    pub status: String,
    /// 参数：tags，标签列表
    pub tags: Vec<String>,
}
