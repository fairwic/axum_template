use axum::{
    routing::{get, post, put},
    Router,
};

use crate::handlers::address_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route(
            "/addresses",
            get(address_handler::list_addresses).post(address_handler::create_address),
        )
        .route(
            "/addresses/:id",
            put(address_handler::update_address).delete(address_handler::delete_address),
        )
        .route(
            "/addresses/:id/set_default",
            post(address_handler::set_default_address),
        )
}
