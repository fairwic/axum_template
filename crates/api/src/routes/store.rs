use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::store_handler;
use crate::state::AppState;

pub fn public_routes() -> Router<AppState> {
    Router::<AppState>::new().route("/stores/nearby", get(store_handler::nearby_stores))
}

pub fn protected_routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/stores/current", get(store_handler::current_store))
        .route("/stores/select", post(store_handler::select_store))
}
