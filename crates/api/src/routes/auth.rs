use axum::{routing::post, Router};

use crate::handlers::auth_handler;

pub fn routes() -> Router {
    Router::new().route("/auth/wechat_login", post(auth_handler::wechat_login))
}
