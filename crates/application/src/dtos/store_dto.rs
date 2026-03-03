//! Store application DTOs

use axum_domain::store::entity::StoreStatus;

/// 应用层输入：创建门店
#[derive(Debug, Clone)]
pub struct CreateStoreInput {
    pub name: String,
    pub address: String,
    pub lat: f64,
    pub lng: f64,
    pub phone: String,
    pub business_hours: String,
    pub status: StoreStatus,
    pub delivery_radius_km: f64,
    pub delivery_fee_base: i32,
    pub delivery_fee_per_km: i32,
    pub runner_service_fee: i32,
}

/// 应用层输入：更新门店
#[derive(Debug, Clone)]
pub struct UpdateStoreInput {
    pub name: String,
    pub address: String,
    pub lat: f64,
    pub lng: f64,
    pub phone: String,
    pub business_hours: String,
    pub status: StoreStatus,
    pub delivery_radius_km: f64,
    pub delivery_fee_base: i32,
    pub delivery_fee_per_km: i32,
    pub runner_service_fee: i32,
}
