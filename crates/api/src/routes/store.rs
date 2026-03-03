use axum::{routing::get, Router};

use crate::handlers::store_handler;

pub fn routes() -> Router {
    Router::new().route("/stores/nearby", get(store_handler::nearby_stores))
}
