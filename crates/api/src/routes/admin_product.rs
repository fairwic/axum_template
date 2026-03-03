use axum::{
    routing::{post, put},
    Router,
};

use crate::handlers::admin_product_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/products",
            post(admin_product_handler::admin_create_product),
        )
        .route(
            "/products/:id",
            put(admin_product_handler::admin_update_product),
        )
}
