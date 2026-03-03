//! Postgres implementation for UserRepository

use axum_common::AppResult;
use axum_domain::user::repo::UserRepository;
use axum_domain::User;
use async_trait::async_trait;
use sqlx::PgPool;
use ulid::Ulid;

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
    async fn create(&self, user: &User) -> AppResult<User> {
        let model = UserModel::from_entity(user);
        let row = sqlx::query_as!(
            UserModel,
            r#"
            INSERT INTO users (id, name, email, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, email, created_at, updated_at
            "#,
            model.id,
            model.name,
            model.email,
            model.created_at,
            model.updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        row.into_entity()
    }

    async fn find_by_id(&self, id: Ulid) -> AppResult<Option<User>> {
        let row = sqlx::query_as!(
            UserModel,
            r#"SELECT id, name, email, created_at, updated_at FROM users WHERE id = $1"#,
            id.to_string()
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(model) => Ok(Some(model.into_entity()?)),
            None => Ok(None),
        }
    }

    async fn list(&self) -> AppResult<Vec<User>> {
        let rows = sqlx::query_as!(
            UserModel,
            r#"SELECT id, name, email, created_at, updated_at FROM users ORDER BY created_at DESC"#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut users = Vec::with_capacity(rows.len());
        for model in rows {
            users.push(model.into_entity()?);
        }
        Ok(users)
    }

    async fn update(&self, user: &User) -> AppResult<User> {
        let row = sqlx::query_as!(
            UserModel,
            r#"
            UPDATE users
            SET name = $2, email = $3, updated_at = $4
            WHERE id = $1
            RETURNING id, name, email, created_at, updated_at
            "#,
            user.id.to_string(),
            &user.name,
            &user.email,
            user.updated_at
        )
        .fetch_one(&self.pool)
        .await?;

        row.into_entity()
    }

    async fn delete(&self, id: Ulid) -> AppResult<bool> {
        let result = sqlx::query!(
            "DELETE FROM users WHERE id = $1",
            id.to_string()
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}
