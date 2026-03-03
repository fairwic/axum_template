use axum::{
    routing::{post, put},
    Router,
};

use crate::handlers::admin_category_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/categories",
            post(admin_category_handler::admin_create_category),
        )
        .route(
            "/categories/:id",
            put(admin_category_handler::admin_update_category),
        )
}
