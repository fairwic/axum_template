//! Backend Template API

pub mod auth;
pub mod handlers;
pub mod openapi;
pub mod router;
pub mod routes;
pub mod state;

pub use router::create_router;
pub use state::AppState;
