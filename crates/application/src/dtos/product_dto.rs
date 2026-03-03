//! Product application DTOs

use axum_domain::product::entity::ProductStatus;
use ulid::Ulid;

/// 应用层输入：创建商品
#[derive(Debug, Clone)]
pub struct CreateProductInput {
    pub store_id: Ulid,
    pub category_id: Ulid,
    pub title: String,
    pub subtitle: Option<String>,
    pub cover_image: String,
    pub images: Vec<String>,
    pub price: i32,
    pub original_price: Option<i32>,
    pub stock: i32,
    pub status: ProductStatus,
    pub tags: Vec<String>,
}

/// 应用层输入：更新商品
#[derive(Debug, Clone)]
pub struct UpdateProductInput {
    pub store_id: Ulid,
    pub category_id: Ulid,
    pub title: String,
    pub subtitle: Option<String>,
    pub cover_image: String,
    pub images: Vec<String>,
    pub price: i32,
    pub original_price: Option<i32>,
    pub stock: i32,
    pub status: ProductStatus,
    pub tags: Vec<String>,
}
