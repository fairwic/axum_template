use std::collections::HashMap;
use std::sync::Arc;

use axum_application::UserService;
use axum_common::AppResult;
use axum_domain::user::repo::UserRepository;
use axum_domain::CacheService;
use axum_domain::User;
use async_trait::async_trait;
use tokio::sync::Mutex;
use ulid::Ulid;

#[derive(Default)]
struct InMemoryUserRepo {
    inner: Mutex<HashMap<Ulid, User>>,
    find_calls: Mutex<usize>,
}

#[async_trait]
impl UserRepository for InMemoryUserRepo {
    async fn create(&self, user: &User) -> AppResult<User> {
        let mut guard = self.inner.lock().await;
        guard.insert(user.id, user.clone());
        Ok(user.clone())
    }

    async fn find_by_id(&self, id: Ulid) -> AppResult<Option<User>> {
        let mut count = self.find_calls.lock().await;
        *count += 1;
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

impl InMemoryUserRepo {
    async fn find_call_count(&self) -> usize {
        let guard = self.find_calls.lock().await;
        *guard
    }
}

#[derive(Default)]
struct InMemoryCache {
    inner: Mutex<HashMap<String, String>>,
}

#[async_trait]
impl CacheService for InMemoryCache {
    async fn get_string(&self, key: &str) -> AppResult<Option<String>> {
        let guard = self.inner.lock().await;
        Ok(guard.get(key).cloned())
    }

    async fn set_string(&self, key: &str, value: &str, _ttl_secs: u64) -> AppResult<()> {
        let mut guard = self.inner.lock().await;
        guard.insert(key.to_string(), value.to_string());
        Ok(())
    }

    async fn delete(&self, key: &str) -> AppResult<()> {
        let mut guard = self.inner.lock().await;
        guard.remove(key);
        Ok(())
    }
}

fn user_cache_key(id: Ulid) -> String {
    format!("user:{id}")
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

#[tokio::test]
async fn test_get_user_reads_cache_first() {
    let repo = Arc::new(InMemoryUserRepo::default());
    let cache = Arc::new(InMemoryCache::default());
    let service = UserService::new_with_cache(repo.clone(), cache.clone(), 300);

    let created = service
        .create_user("Cache".into(), "cache@b.com".into())
        .await
        .unwrap();

    let cache_key = user_cache_key(created.id);
    let cached = serde_json::to_string(&created).unwrap();
    cache.set_string(&cache_key, &cached, 300).await.unwrap();

    let fetched = service.get_user(created.id).await.unwrap();
    assert_eq!(fetched, created);

    let calls = repo.find_call_count().await;
    assert_eq!(calls, 0, "expected repo to be skipped due to cache hit");
}

#[tokio::test]
async fn test_get_user_cache_miss_writes_cache() {
    let repo = Arc::new(InMemoryUserRepo::default());
    let cache = Arc::new(InMemoryCache::default());
    let service = UserService::new_with_cache(repo.clone(), cache.clone(), 300);

    let created = service
        .create_user("Miss".into(), "miss@b.com".into())
        .await
        .unwrap();

    let fetched = service.get_user(created.id).await.unwrap();
    assert_eq!(fetched, created);

    let calls = repo.find_call_count().await;
    assert_eq!(calls, 1, "expected repo call on cache miss");

    let cache_key = user_cache_key(created.id);
    let cached = cache.get_string(&cache_key).await.unwrap();
    assert!(cached.is_some(), "expected cache to be populated after miss");
}
