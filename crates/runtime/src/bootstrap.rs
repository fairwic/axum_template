use std::sync::Arc;

use sqlx::{Pool, Postgres};

use axum_api::state::AppState;
use axum_application::AddressService;
use axum_infra::{AppConfig, PgAddressRepository};

pub async fn build_app_state(pool: Pool<Postgres>, config: &AppConfig) -> anyhow::Result<AppState> {
    let address_repo = Arc::new(PgAddressRepository::new(pool.clone()));

    let address_service = AddressService::new(address_repo);

    Ok(AppState::new(
        address_service,
        config.auth.jwt_secret.clone(),
        config.auth.jwt_ttl_secs,
        config.sms.login_code_ttl_secs,
    ))
}
