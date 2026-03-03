use axum::{routing::get, Router};

use crate::handlers::member_handler;

pub fn routes() -> Router {
    Router::new()
        .route("/member/status", get(member_handler::member_status))
        .route("/member/benefits", get(member_handler::member_benefits))
}
