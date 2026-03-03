use std::sync::Arc;

use sqlx::{Pool, Postgres};

use axum_api::state::AppState;
use axum_application::{AdminService, UserService};
use axum_infrastructure::{AppConfig, PgAdminRepository, PgUserRepository};

/// Build AppState with minimal dependencies
pub async fn build_app_state(pool: Pool<Postgres>, config: &AppConfig) -> anyhow::Result<AppState> {
    let user_repo = Arc::new(PgUserRepository::new(pool.clone()));
    let admin_repo = Arc::new(PgAdminRepository::new(pool));
    let user_service = UserService::new(user_repo);
    let admin_service = AdminService::new(admin_repo);

    Ok(AppState::new(
        user_service,
        admin_service,
        config.auth.jwt_secret.clone(),
        config.auth.jwt_ttl_secs,
    ))
}
