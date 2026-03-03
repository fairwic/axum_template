use axum::{routing::get, Router};

use crate::handlers::config_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new().route("/config", get(config_handler::get_config))
}
