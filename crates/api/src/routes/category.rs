use axum::{routing::get, Router};

use crate::handlers::category_handler;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::<AppState>::new().route("/categories", get(category_handler::list_categories))
}
