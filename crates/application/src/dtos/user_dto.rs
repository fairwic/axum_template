//! User DTOs

use axum_domain::User;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct WechatLoginDto {
    pub openid: String,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub openid: String,
    pub nickname: Option<String>,
    pub avatar: Option<String>,
    pub phone: Option<String>,
    pub is_member: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            openid: user.openid,
            nickname: user.nickname,
            avatar: user.avatar,
            phone: user.phone,
            is_member: user.is_member,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
