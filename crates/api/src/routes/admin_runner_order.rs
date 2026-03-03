use axum::{routing::{get, post}, Router};

use crate::handlers::admin_runner_order_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/runner_orders", get(admin_runner_order_handler::admin_list_runner_orders))
        .route("/runner_orders/:id/accept", post(admin_runner_order_handler::admin_accept_runner_order))
        .route("/runner_orders/:id/delivered", post(admin_runner_order_handler::admin_delivered_runner_order))
        .route("/runner_orders/:id/complete", post(admin_runner_order_handler::admin_complete_runner_order))
}
