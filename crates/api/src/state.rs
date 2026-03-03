use std::sync::Arc;

use axum_application::{AdminService, UserService};

#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<UserService>,
    pub admin_service: Arc<AdminService>,
    pub jwt_secret: String,
    pub jwt_ttl_secs: u64,
}

impl AppState {
    pub fn new(
        user_service: UserService,
        admin_service: AdminService,
        jwt_secret: String,
        jwt_ttl_secs: u64,
    ) -> Self {
        Self {
            user_service: Arc::new(user_service),
            admin_service: Arc::new(admin_service),
            jwt_secret,
            jwt_ttl_secs,
        }
    }
}
