//! User service

use std::sync::Arc;

use axum_common::{AppError, AppResult};
use axum_domain::user::repo::UserRepository;
use axum_domain::CacheService;
use axum_domain::User;
use ulid::Ulid;

#[derive(Clone)]
pub struct UserService {
    repo: Arc<dyn UserRepository>,
    cache: Option<Arc<dyn CacheService>>,
    cache_ttl_secs: u64,
}

impl UserService {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self {
            repo,
            cache: None,
            cache_ttl_secs: 0,
        }
    }

    pub fn new_with_cache(
        repo: Arc<dyn UserRepository>,
        cache: Arc<dyn CacheService>,
        cache_ttl_secs: u64,
    ) -> Self {
        Self {
            repo,
            cache: Some(cache),
            cache_ttl_secs,
        }
    }

    pub async fn create_user(&self, name: String, email: String) -> AppResult<User> {
        let user = User::new(name, email)?;
        let created = self.repo.create(&user).await?;
        self.invalidate_cache(created.id).await;
        Ok(created)
    }

    pub async fn get_user(&self, id: Ulid) -> AppResult<User> {
        if let Some(cache) = &self.cache {
            match cache.get_string(&Self::cache_key(id)).await {
                Ok(Some(cached)) => match serde_json::from_str::<User>(&cached) {
                    Ok(user) => return Ok(user),
                    Err(err) => {
                        tracing::warn!(error = %err, "Failed to deserialize cached user");
                        if let Err(err) = cache.delete(&Self::cache_key(id)).await {
                            tracing::warn!(error = %err, "Failed to evict corrupted cache entry");
                        }
                    }
                },
                Ok(None) => {}
                Err(err) => tracing::warn!(error = %err, "Cache get failed"),
            }
        }

        let user = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".into()))?;

        self.set_cache(&user).await;
        Ok(user)
    }

    pub async fn list_users(&self) -> AppResult<Vec<User>> {
        self.repo.list().await
    }

    pub async fn update_user(&self, id: Ulid, name: String, email: String) -> AppResult<User> {
        let mut user = self.get_user(id).await?;
        user.update(name, email)?;
        let updated = self.repo.update(&user).await?;
        self.invalidate_cache(id).await;
        Ok(updated)
    }

    pub async fn delete_user(&self, id: Ulid) -> AppResult<()> {
        let deleted = self.repo.delete(id).await?;
        if !deleted {
            return Err(AppError::NotFound("user not found".into()));
        }
        self.invalidate_cache(id).await;
        Ok(())
    }

    fn cache_key(id: Ulid) -> String {
        format!("user:{id}")
    }

    async fn set_cache(&self, user: &User) {
        let Some(cache) = &self.cache else {
            return;
        };

        let key = Self::cache_key(user.id);
        let payload = match serde_json::to_string(user) {
            Ok(payload) => payload,
            Err(err) => {
                tracing::warn!(error = %err, "Failed to serialize user for cache");
                return;
            }
        };

        if let Err(err) = cache.set_string(&key, &payload, self.cache_ttl_secs).await {
            tracing::warn!(error = %err, "Cache set failed");
        }
    }

    async fn invalidate_cache(&self, id: Ulid) {
        let Some(cache) = &self.cache else {
            return;
        };

        if let Err(err) = cache.delete(&Self::cache_key(id)).await {
            tracing::warn!(error = %err, "Cache delete failed");
        }
    }
}
