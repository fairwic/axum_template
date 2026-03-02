use std::sync::Arc;

use sqlx::{Pool, Postgres};

use axum_api::state::AppState;
use axum_application::UserService;
use axum_infrastructure::PgUserRepository;

/// Build AppState with minimal dependencies
pub async fn build_app_state(pool: Pool<Postgres>) -> anyhow::Result<AppState> {
    let user_repo = Arc::new(PgUserRepository::new(pool));
    let user_service = UserService::new(user_repo);

    Ok(AppState::new(user_service))
}
