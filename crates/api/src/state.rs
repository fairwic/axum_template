use std::sync::Arc;

use axum_application::snapshot::ingest_service::IngestService;
use axum_application::AddressService;
use axum_core_kernel::{AppError, AppResult};

/// 应用共享状态，随路由注入各 handler。
#[derive(Clone)]
pub struct AppState {
    pub address_service: Option<Arc<AddressService>>,
    pub ingest_service: Option<Arc<IngestService>>,
    pub jwt_secret: String,
    pub jwt_ttl_secs: u64,
    pub sms_code_ttl_secs: u64,
}

impl AppState {
    pub fn address_service_ref(&self) -> AppResult<&Arc<AddressService>> {
        self.address_service
            .as_ref()
            .ok_or_else(|| AppError::Internal("address_service is not configured".into()))
    }

    /// 构造应用状态，注入地址服务与鉴权相关配置。
    pub fn new(
        address_service: AddressService,
        jwt_secret: String,
        jwt_ttl_secs: u64,
        sms_code_ttl_secs: u64,
    ) -> Self {
        Self {
            address_service: Some(Arc::new(address_service)),
            ingest_service: None,
            jwt_secret,
            jwt_ttl_secs,
            sms_code_ttl_secs,
        }
    }

    /// 链式注入快照摄取服务。
    pub fn with_ingest_service(mut self, service: IngestService) -> Self {
        self.ingest_service = Some(Arc::new(service));
        self
    }
}

impl axum::extract::FromRef<AppState> for Arc<IngestService> {
    fn from_ref(state: &AppState) -> Self {
        state
            .ingest_service
            .clone()
            .expect("ingest_service is not configured")
    }
}
