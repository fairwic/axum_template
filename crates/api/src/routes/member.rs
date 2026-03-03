use axum::{routing::get, Router};

use crate::handlers::member_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/member/status", get(member_handler::member_status))
        .route("/member/benefits", get(member_handler::member_benefits))
}
