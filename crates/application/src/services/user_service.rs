//! User service

use std::sync::Arc;

use axum_common::{AppError, AppResult};
use axum_domain::user::repo::UserRepository;
use axum_domain::User;
use ulid::Ulid;

#[derive(Clone)]
pub struct UserService {
    repo: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_user(&self, name: String, email: String) -> AppResult<User> {
        let user = User::new(name, email)?;
        self.repo.create(&user).await
    }

    pub async fn get_user(&self, id: Ulid) -> AppResult<User> {
        let user = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".into()))?;
        Ok(user)
    }

    pub async fn list_users(&self) -> AppResult<Vec<User>> {
        self.repo.list().await
    }

    pub async fn update_user(&self, id: Ulid, name: String, email: String) -> AppResult<User> {
        let mut user = self.get_user(id).await?;
        user.update(name, email)?;
        self.repo.update(&user).await
    }

    pub async fn delete_user(&self, id: Ulid) -> AppResult<()> {
        let deleted = self.repo.delete(id).await?;
        if !deleted {
            return Err(AppError::NotFound("user not found".into()));
        }
        Ok(())
    }
}
