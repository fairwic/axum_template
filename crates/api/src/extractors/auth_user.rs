use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_common::AppError;
use ulid::Ulid;

use crate::auth::jwt::Claims;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Ulid,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = parts
            .extensions
            .get::<Claims>()
            .cloned()
            .ok_or(AppError::Unauthorized)?;
        if claims.role != "USER" {
            return Err(AppError::Forbidden);
        }
        let user_id = Ulid::from_string(&claims.sub)
            .map_err(|_| AppError::Validation("invalid user_id".into()))?;
        Ok(Self { user_id })
    }
}
