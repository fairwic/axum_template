use axum::{routing::get, Router};

use crate::handlers::category_handler;

pub fn routes() -> Router {
    Router::new().route("/categories", get(category_handler::list_categories))
}
