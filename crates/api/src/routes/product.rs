use axum::{routing::get, Router};

use crate::handlers::product_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/products", get(product_handler::list_products))
        .route("/products/search", get(product_handler::search_products))
        .route("/products/:id", get(product_handler::get_product))
}
