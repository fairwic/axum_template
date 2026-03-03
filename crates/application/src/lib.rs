//! # Backend Template Application
//!
//! 应用层，负责业务编排与用例输入输出定义。

pub mod dtos {
    pub mod address_dto;
    pub mod order_dto;
    pub mod runner_order_dto;
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

pub use dtos::address_dto::{CreateAddressInput, UpdateAddressInput};
pub use dtos::order_dto::{CreateGoodsOrderInput, OrderPreview};
pub use dtos::runner_order_dto::CreateRunnerOrderInput;
pub use services::address_service::AddressService;
pub use services::admin_service::AdminService;
pub use services::cart_service::CartService;
pub use services::category_service::CategoryService;
pub use services::order_service::OrderService;
pub use services::product_service::ProductService;
pub use services::runner_order_service::RunnerOrderService;
pub use services::store_service::StoreService;
pub use services::user_service::UserService;
