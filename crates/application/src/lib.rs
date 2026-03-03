//! # Backend Template Application
//!
//! 应用层，仅保留最小 UserService 示例。

pub mod dtos {
    pub mod user_dto;
}

pub mod services {
    pub mod address_service;
    pub mod admin_service;
    pub mod cart_service;
    pub mod category_service;
    pub mod order_service;
    pub mod product_service;
    pub mod runner_order_service;
    pub mod store_service;
    pub mod user_service;
}

pub use services::address_service::{AddressService, CreateAddressInput, UpdateAddressInput};
pub use services::admin_service::AdminService;
pub use services::cart_service::CartService;
pub use services::category_service::CategoryService;
pub use services::order_service::{CreateGoodsOrderInput, OrderPreview, OrderService};
pub use services::product_service::ProductService;
pub use services::runner_order_service::{CreateRunnerOrderInput, RunnerOrderService};
pub use services::store_service::StoreService;
pub use services::user_service::UserService;
