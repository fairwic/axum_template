use std::sync::Arc;

use sqlx::{Pool, Postgres};

use axum_api::state::AppState;
use axum_application::snapshot::ingest_service::IngestService;
use axum_application::AddressService;
use axum_infra::{
    AppConfig, NoopEventPublisher, PgAddressRepository, ScyllaHotStore, TemuAdapter, YandexAdapter,
};

pub async fn build_app_state(pool: Pool<Postgres>, config: &AppConfig) -> anyhow::Result<AppState> {
    // 地址服务
    let address_repo = Arc::new(PgAddressRepository::new(pool.clone()));
    let address_service = AddressService::new(address_repo);

    // 快照热存储（ScyllaDB）
    let hot_store = Arc::new(
        ScyllaHotStore::from_config(config)
            .await
            .map_err(|e| anyhow::anyhow!("ScyllaDB 连接失败: {e}"))?,
    );

    // 事件发布（Noop，不依赖 MQ）
    let publisher = Arc::new(NoopEventPublisher::default());

    // 平台适配器
    let adapters: Vec<Arc<dyn axum_domain::snapshot::ports::PlatformSnapshotAdapter>> =
        vec![Arc::new(TemuAdapter::new()), Arc::new(YandexAdapter::new())];

    let ingest_service = IngestService::new(hot_store, publisher, adapters);

    Ok(AppState::new(
        address_service,
        config.auth.jwt_secret.clone(),
        config.auth.jwt_ttl_secs,
        config.sms.login_code_ttl_secs,
    )
    .with_ingest_service(ingest_service))
}
