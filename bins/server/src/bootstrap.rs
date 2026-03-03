use std::sync::Arc;

use sqlx::{Pool, Postgres};

use axum_api::state::AppState;
use axum_application::{AdminService, CategoryService, ProductService, StoreService, UserService};
use axum_application::services::store_service::LbsService;
use axum_infrastructure::{AppConfig, PgAdminRepository, PgCategoryRepository, PgProductRepository, PgStoreRepository, PgUserRepository};
use async_trait::async_trait;

/// Build AppState with minimal dependencies
pub async fn build_app_state(pool: Pool<Postgres>, config: &AppConfig) -> anyhow::Result<AppState> {
    let user_repo = Arc::new(PgUserRepository::new(pool.clone()));
    let store_repo = Arc::new(PgStoreRepository::new(pool.clone()));
    let category_repo = Arc::new(PgCategoryRepository::new(pool.clone()));
    let product_repo = Arc::new(PgProductRepository::new(pool.clone()));
    let admin_repo = Arc::new(PgAdminRepository::new(pool));
    let user_service = UserService::new(user_repo);
    let admin_service = AdminService::new(admin_repo);
    let store_service = StoreService::new(store_repo, Arc::new(NoopLbs));
    let category_service = CategoryService::new(category_repo);
    let product_service = ProductService::new(product_repo);

    Ok(AppState::new(
        user_service,
        admin_service,
        store_service,
        category_service,
        product_service,
        config.auth.jwt_secret.clone(),
        config.auth.jwt_ttl_secs,
    ))
}

struct NoopLbs;

#[async_trait]
impl LbsService for NoopLbs {
    async fn distance_km(&self, _from: (f64, f64), _to: (f64, f64)) -> axum_common::AppResult<f64> {
        Ok(0.0)
    }
}
