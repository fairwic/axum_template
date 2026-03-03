//! User service

use std::sync::Arc;

use axum_common::AppResult;
use axum_domain::user::repo::UserRepository;
use axum_domain::User;

#[derive(Clone)]
pub struct UserService {
    repo: Arc<dyn UserRepository>,
}

impl UserService {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn login_with_openid(
        &self,
        openid: String,
        nickname: Option<String>,
        avatar: Option<String>,
    ) -> AppResult<User> {
        if let Some(user) = self.repo.find_by_openid(&openid).await? {
            return Ok(user);
        }
        let user = User::new(openid, nickname, avatar)?;
        self.repo.create(&user).await
    }
}
