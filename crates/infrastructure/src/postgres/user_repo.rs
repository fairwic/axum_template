//! Postgres implementation for UserRepository

use axum_common::AppResult;
use axum_domain::user::repo::UserRepository;
use axum_domain::User;
use async_trait::async_trait;
use sqlx::PgPool;

use crate::models::user_model::UserModel;

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn find_by_openid(&self, openid: &str) -> AppResult<Option<User>> {
        let row = sqlx::query_as!(
            UserModel,
            r#"
            SELECT id, openid, nickname, avatar, phone, is_member, created_at, updated_at
            FROM users
            WHERE openid = $1
            "#,
            openid
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(model) => Ok(Some(model.into_entity()?)),
            None => Ok(None),
        }
    }

    async fn create(&self, user: &User) -> AppResult<User> {
        let model = UserModel::from_entity(user);
        let row = sqlx::query_as!(
            UserModel,
            r#"
            INSERT INTO users (id, openid, nickname, avatar, phone, is_member, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, openid, nickname, avatar, phone, is_member, created_at, updated_at
            "#,
            model.id,
            model.openid,
            model.nickname,
            model.avatar,
            model.phone,
            model.is_member,
            model.created_at,
            model.updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        row.into_entity()
    }
}
