use std::sync::Arc;

use axum_application::UserService;

#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<UserService>,
    pub jwt_secret: String,
    pub jwt_ttl_secs: u64,
}

impl AppState {
    pub fn new(user_service: UserService, jwt_secret: String, jwt_ttl_secs: u64) -> Self {
        Self {
            user_service: Arc::new(user_service),
            jwt_secret,
            jwt_ttl_secs,
        }
    }
}
