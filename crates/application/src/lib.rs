//! # Backend Template Application
//!
//! 应用层，负责业务编排与用例输入输出定义。

pub mod snapshot;
pub mod dtos {
    pub mod address_dto;
}

pub mod services {
    pub mod address_service;
}

pub use dtos::address_dto::{CreateAddressInput, UpdateAddressInput};
pub use services::address_service::AddressService;
