use std::sync::Arc;

use sqlx::{Pool, Postgres};

use axum_api::state::AppState;
use axum_application::UserService;
use axum_infrastructure::{AppConfig, PgUserRepository, RedisCacheService};

/// Build AppState with minimal dependencies
pub async fn build_app_state(pool: Pool<Postgres>, config: &AppConfig) -> anyhow::Result<AppState> {
    let user_repo = Arc::new(PgUserRepository::new(pool));
    let cache = RedisCacheService::new(&config.redis.url, config.redis.max_connections).await?;
    let user_service =
        UserService::new_with_cache(user_repo, Arc::new(cache), config.cache.default_ttl_secs);

    Ok(AppState::new(user_service))
}
