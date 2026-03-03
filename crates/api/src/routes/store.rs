use axum::{routing::get, Router};

use crate::handlers::store_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new().route("/stores/nearby", get(store_handler::nearby_stores))
}
