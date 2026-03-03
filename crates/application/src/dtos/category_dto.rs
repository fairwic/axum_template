//! Category application DTOs

use axum_domain::category::entity::CategoryStatus;
use ulid::Ulid;

/// 应用层输入：创建类目
#[derive(Debug, Clone)]
pub struct CreateCategoryInput {
    pub store_id: Ulid,
    pub name: String,
    pub sort_order: i32,
    pub status: CategoryStatus,
}

/// 应用层输入：更新类目
#[derive(Debug, Clone)]
pub struct UpdateCategoryInput {
    pub name: String,
    pub sort_order: i32,
    pub status: CategoryStatus,
}
