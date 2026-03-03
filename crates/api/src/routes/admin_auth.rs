use axum::{routing::post, Router};

use crate::handlers::admin_auth_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new().route("/auth/login", post(admin_auth_handler::admin_login))
}
