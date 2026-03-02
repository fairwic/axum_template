//! User DTOs

use axum_domain::User;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserDto {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserDto {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            name: user.name,
            email: user.email,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
