use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::runner_order_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/runner_orders/create",
            post(runner_order_handler::create_runner_order),
        )
        .route(
            "/runner_orders/pay",
            post(runner_order_handler::pay_runner_order),
        )
        .route(
            "/runner_orders",
            get(runner_order_handler::list_runner_orders),
        )
        .route(
            "/runner_orders/:id",
            get(runner_order_handler::get_runner_order),
        )
        .route(
            "/runner_orders/:id/cancel",
            post(runner_order_handler::cancel_runner_order),
        )
}
