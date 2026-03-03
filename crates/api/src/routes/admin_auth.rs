use axum::{routing::post, Router};

use crate::handlers::admin_auth_handler;

pub fn routes() -> Router {
    Router::new().route("/auth/login", post(admin_auth_handler::admin_login))
}
