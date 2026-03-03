use axum::{
    routing::{get, put},
    Router,
};

use crate::handlers::admin_store_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/stores",
            get(admin_store_handler::admin_list_stores)
                .post(admin_store_handler::admin_create_store),
        )
        .route("/stores/:id", put(admin_store_handler::admin_update_store))
}
