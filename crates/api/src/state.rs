use std::sync::Arc;

use axum_application::UserService;

#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<UserService>,
}

impl AppState {
    pub fn new(user_service: UserService) -> Self {
        Self {
            user_service: Arc::new(user_service),
        }
    }
}
