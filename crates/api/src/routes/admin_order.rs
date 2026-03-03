use axum::{routing::{get, post}, Router};

use crate::handlers::admin_order_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/orders", get(admin_order_handler::admin_list_orders))
        .route("/orders/:id/accept", post(admin_order_handler::admin_accept_order))
        .route("/orders/:id/dispatch", post(admin_order_handler::admin_dispatch_order))
        .route("/orders/:id/complete", post(admin_order_handler::admin_complete_order))
}
