use std::collections::HashMap;
use std::sync::Arc;

use axum::{body::{Body, to_bytes}, http::Request};
use axum_api::{create_router, AppState};
use axum_application::{AdminService, UserService};
use axum_common::AppResult;
use axum_domain::admin::entity::{Admin, AdminRole};
use axum_domain::admin::repo::AdminRepository;
use axum_domain::user::repo::UserRepository;
use axum_domain::User;
use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::Mutex;
use tower::util::ServiceExt;

#[derive(Default)]
struct InMemoryUserRepo {
    inner: Mutex<HashMap<String, User>>,
}

#[async_trait]
impl UserRepository for InMemoryUserRepo {
    async fn find_by_openid(&self, openid: &str) -> AppResult<Option<User>> {
        let guard = self.inner.lock().await;
        Ok(guard.get(openid).cloned())
    }

    async fn create(&self, user: &User) -> AppResult<User> {
        let mut guard = self.inner.lock().await;
        guard.insert(user.openid.clone(), user.clone());
        Ok(user.clone())
    }
}

#[derive(Default)]
struct InMemoryAdminRepo {
    inner: Mutex<HashMap<String, Admin>>,
}

#[async_trait]
impl AdminRepository for InMemoryAdminRepo {
    async fn find_by_phone(&self, phone: &str) -> AppResult<Option<Admin>> {
        let guard = self.inner.lock().await;
        Ok(guard.get(phone).cloned())
    }

    async fn create(&self, admin: &Admin) -> AppResult<Admin> {
        let mut guard = self.inner.lock().await;
        guard.insert(admin.phone.clone(), admin.clone());
        Ok(admin.clone())
    }
}

#[tokio::test]
async fn test_admin_login_returns_token() {
    let user_repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepo::default());
    let admin_repo: Arc<dyn AdminRepository> = Arc::new(InMemoryAdminRepo::default());
    let user_service = UserService::new(user_repo);
    let admin_service = AdminService::new(admin_repo.clone());

    admin_service
        .create_admin("13800000000".into(), "pass".into(), AdminRole::Platform, None)
        .await
        .unwrap();

    let state = AppState::new(user_service, admin_service, "secret".into(), 3600);
    let app = create_router(state);

    let req = Request::builder()
        .method("POST")
        .uri("/api/admin/v1/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"phone":"13800000000","password":"pass"}"#))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    let body = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let value: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["success"], true);
    assert!(value["data"]["token"].as_str().unwrap_or("").len() > 0);
    assert_eq!(value["data"]["admin"]["phone"], "13800000000");
}
