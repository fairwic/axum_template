use std::collections::HashMap;
use std::sync::Arc;

use axum_application::UserService;
use axum_common::AppResult;
use axum_domain::user::repo::UserRepository;
use axum_domain::User;
use async_trait::async_trait;
use tokio::sync::Mutex;

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

#[tokio::test]
async fn test_login_creates_user() {
    let repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepo::default());
    let service = UserService::new(repo);

    let user = service
        .login_with_openid("openid-1".into(), None, None)
        .await
        .unwrap();

    assert_eq!(user.openid, "openid-1");
    assert!(user.is_member);
}

#[tokio::test]
async fn test_login_returns_existing_user() {
    let repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepo::default());
    let service = UserService::new(repo);

    let first = service
        .login_with_openid("openid-2".into(), Some("Nick".into()), None)
        .await
        .unwrap();

    let second = service
        .login_with_openid("openid-2".into(), Some("Other".into()), None)
        .await
        .unwrap();

    assert_eq!(first.id, second.id);
    assert_eq!(second.openid, "openid-2");
}
