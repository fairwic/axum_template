use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::user_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/users",
            post(user_handler::create_user).get(user_handler::list_users),
        )
        .route(
            "/users/:id",
            get(user_handler::get_user)
                .put(user_handler::update_user)
                .delete(user_handler::delete_user),
        )
}
