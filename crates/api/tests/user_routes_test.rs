use std::collections::HashMap;
use std::sync::Arc;

use axum::{body::{Body, to_bytes}, http::{Request, StatusCode}};
use axum_api::{create_router, AppState};
use axum_application::UserService;
use axum_common::AppResult;
use axum_domain::user::repo::UserRepository;
use axum_domain::User;
use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::Mutex;
use tower::util::ServiceExt;
use ulid::Ulid;

#[derive(Default)]
struct InMemoryUserRepo {
    inner: Mutex<HashMap<Ulid, User>>,
}

#[async_trait]
impl UserRepository for InMemoryUserRepo {
    async fn create(&self, user: &User) -> AppResult<User> {
        let mut guard = self.inner.lock().await;
        guard.insert(user.id, user.clone());
        Ok(user.clone())
    }

    async fn find_by_id(&self, id: Ulid) -> AppResult<Option<User>> {
        let guard = self.inner.lock().await;
        Ok(guard.get(&id).cloned())
    }

    async fn list(&self) -> AppResult<Vec<User>> {
        let guard = self.inner.lock().await;
        Ok(guard.values().cloned().collect())
    }

    async fn update(&self, user: &User) -> AppResult<User> {
        let mut guard = self.inner.lock().await;
        guard.insert(user.id, user.clone());
        Ok(user.clone())
    }

    async fn delete(&self, id: Ulid) -> AppResult<bool> {
        let mut guard = self.inner.lock().await;
        Ok(guard.remove(&id).is_some())
    }
}

fn create_test_app() -> axum::Router {
    let repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepo::default());
    let service = UserService::new(repo);
    let state = AppState::new(service);
    create_router(state)
}

#[tokio::test]
async fn test_create_and_get_user_route() {
    let app = create_test_app();

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/users")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"name":"Alice","email":"a@b.com"}"#))
        .unwrap();

    let res = app.clone().oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let value: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(value["success"], true);
    let id = value["data"]["id"].as_str().unwrap().to_string();

    let res = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/v1/users/{}", id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}
