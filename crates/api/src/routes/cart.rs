use axum::{routing::{get, post}, Router};

use crate::handlers::cart_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/cart", get(cart_handler::get_cart))
        .route("/cart/add", post(cart_handler::add_item))
        .route("/cart/update_qty", post(cart_handler::update_qty))
        .route("/cart/remove", post(cart_handler::remove_item))
        .route("/cart/clear", post(cart_handler::clear_cart))
}
