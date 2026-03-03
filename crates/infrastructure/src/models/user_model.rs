//! User model for persistence

use axum_common::AppError;
use axum_domain::User;
use chrono::{DateTime, Utc};
use ulid::Ulid;

#[derive(Debug, sqlx::FromRow)]
pub struct UserModel {
    pub id: String,
    pub openid: String,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
    pub phone: Option<String>,
    pub is_member: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserModel {
    pub fn from_entity(user: &User) -> Self {
        Self {
            id: user.id.to_string(),
            openid: user.openid.clone(),
            nickname: user.nickname.clone(),
            avatar: user.avatar.clone(),
            phone: user.phone.clone(),
            is_member: user.is_member,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }

    pub fn into_entity(self) -> Result<User, AppError> {
        let id = Ulid::from_string(&self.id)
            .map_err(|_| AppError::Internal("invalid ulid in database".into()))?;
        Ok(User {
            id,
            openid: self.openid,
            nickname: self.nickname,
            avatar: self.avatar,
            phone: self.phone,
            is_member: self.is_member,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
