use axum::{routing::get, routing::put, Router};

use crate::handlers::config_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/config", get(config_handler::admin_get_config))
        .route("/config", put(config_handler::admin_update_config))
}
