use std::sync::Arc;

use axum_application::AddressService;
use axum_core_kernel::{AppError, AppResult};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct BizConfig {
    pub delivery_free_radius_km: f64,
    pub runner_service_fee: i32,
    pub customer_service_phone: String,
    pub runner_banner_enabled: bool,
    pub runner_banner_text: String,
    pub pay_timeout_secs: u64,
    pub auto_accept_secs: u64,
    pub cancel_timeout_secs: u64,
}

impl Default for BizConfig {
    fn default() -> Self {
        Self {
            delivery_free_radius_km: 3.0,
            runner_service_fee: 200,
            customer_service_phone: "400-000-0000".into(),
            runner_banner_enabled: true,
            runner_banner_text: "顺路代取快递".into(),
            pay_timeout_secs: 15 * 60,
            auto_accept_secs: 5 * 60,
            cancel_timeout_secs: 5 * 60,
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub address_service: Option<Arc<AddressService>>,
    pub jwt_secret: String,
    pub jwt_ttl_secs: u64,
    pub sms_code_ttl_secs: u64,
    pub biz_config: Arc<RwLock<BizConfig>>,
}

impl AppState {
    pub fn address_service_ref(&self) -> AppResult<&Arc<AddressService>> {
        self.address_service
            .as_ref()
            .ok_or_else(|| AppError::Internal("address_service is not configured".into()))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        address_service: AddressService,
        jwt_secret: String,
        jwt_ttl_secs: u64,
        sms_code_ttl_secs: u64,
    ) -> Self {
        Self {
            address_service: Some(Arc::new(address_service)),
            jwt_secret,
            jwt_ttl_secs,
            sms_code_ttl_secs,
            biz_config: Arc::new(RwLock::new(BizConfig::default())),
        }
    }

    pub fn with_jwt_config(
        self,
        jwt_secret: String,
        jwt_ttl_secs: u64,
        sms_code_ttl_secs: u64,
    ) -> Self {
        Self {
            address_service: None,
            jwt_secret,
            jwt_ttl_secs,
            sms_code_ttl_secs,
            biz_config: Arc::new(RwLock::new(BizConfig::default())),
        }
    }
}
