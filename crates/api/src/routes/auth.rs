use axum::{routing::post, Router};

use crate::handlers::auth_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new().route("/auth/wechat_login", post(auth_handler::wechat_login))
}
