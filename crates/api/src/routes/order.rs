use axum::{routing::{get, post}, Router};

use crate::handlers::order_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/orders/create", post(order_handler::create_order))
        .route("/orders/pay", post(order_handler::pay_order))
        .route("/orders", get(order_handler::list_orders))
        .route("/orders/:id", get(order_handler::get_order))
        .route("/orders/:id/cancel", post(order_handler::cancel_order))
}
