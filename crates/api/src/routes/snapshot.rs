use axum::{routing::post, Router};

use crate::handlers::snapshot_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/ingest/:platform/product",
            post(snapshot_handler::ingest_product),
        )
        .route(
            "/ingest/:platform/shop",
            post(snapshot_handler::ingest_shop),
        )
}
