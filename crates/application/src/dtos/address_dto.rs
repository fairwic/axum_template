//! Address application DTOs

/// 应用层输入：创建收货地址
#[derive(Debug, Clone)]
pub struct CreateAddressInput {
    pub name: String,
    pub phone: String,
    pub detail: String,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub is_default: bool,
}

/// 应用层输入：更新收货地址
#[derive(Debug, Clone)]
pub struct UpdateAddressInput {
    pub name: String,
    pub phone: String,
    pub detail: String,
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub is_default: bool,
}
