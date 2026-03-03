use std::collections::HashMap;
use std::sync::Arc;

use axum::{body::{Body, to_bytes}, http::Request};
use axum_api::{create_router, AppState};
use axum_application::UserService;
use axum_common::AppResult;
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

fn create_test_app() -> axum::Router {
    let repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepo::default());
    let service = UserService::new(repo);
    let state = AppState::new(service, "secret".into(), 3600);
    create_router(state)
}

#[tokio::test]
async fn test_wechat_login_returns_token() {
    let app = create_test_app();

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/wechat_login")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"openid":"openid-1","nickname":"Alice","avatar":null}"#,
        ))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    let body = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let value: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["success"], true);
    assert!(value["data"]["token"].as_str().unwrap_or("").len() > 0);
    assert_eq!(value["data"]["user"]["openid"], "openid-1");
}
