use axum::{routing::get, Router};

use crate::handlers::product_handler;

pub fn routes() -> Router {
    Router::new()
        .route("/products", get(product_handler::list_products))
        .route("/products/search", get(product_handler::search_products))
}
