use std::collections::HashMap;
use std::sync::Arc;

use axum_application::UserService;
use axum_common::AppResult;
use axum_domain::user::repo::UserRepository;
use axum_domain::User;
use async_trait::async_trait;
use tokio::sync::Mutex;
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

#[tokio::test]
async fn test_create_and_get_user() {
    let repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepo::default());
    let service = UserService::new(repo);

    let created = service
        .create_user("Alice".into(), "a@b.com".into())
        .await
        .unwrap();

    let fetched = service.get_user(created.id).await.unwrap();
    assert_eq!(fetched.name, "Alice");
    assert_eq!(fetched.email, "a@b.com");
}
