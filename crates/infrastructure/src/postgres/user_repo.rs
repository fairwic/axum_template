//! Postgres implementation for UserRepository

use async_trait::async_trait;
use axum_common_infra::{map_sqlx_error, map_unique_violation};
use axum_core_kernel::AppResult;
use axum_domain::user::repo::UserRepository;
use axum_domain::User;
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
    async fn find_by_openid(&self, openid: &str) -> AppResult<Option<User>> {
        let row = sqlx::query_as!(
            UserModel,
            r#"
            SELECT id, openid, nickname, avatar, phone, current_store_id, is_member, created_at, updated_at
            FROM users
            WHERE openid = $1
            "#,
            openid
        )
        .fetch_optional(&self.pool)
        .await.map_err(map_sqlx_error)?;

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
            INSERT INTO users (id, openid, nickname, avatar, phone, current_store_id, is_member, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, openid, nickname, avatar, phone, current_store_id, is_member, created_at, updated_at
            "#,
            model.id,
            model.openid,
            model.nickname,
            model.avatar,
            model.phone,
            model.current_store_id,
            model.is_member,
            model.created_at,
            model.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| map_unique_violation(err, "手机号已绑定其他账号"))?;

        row.into_entity()
    }

    async fn find_by_id(&self, user_id: Ulid) -> AppResult<Option<User>> {
        let row = sqlx::query_as!(
            UserModel,
            r#"
            SELECT id, openid, nickname, avatar, phone, current_store_id, is_member, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            user_id.to_string()
        )
        .fetch_optional(&self.pool)
        .await.map_err(map_sqlx_error)?;

        match row {
            Some(model) => Ok(Some(model.into_entity()?)),
            None => Ok(None),
        }
    }

    async fn set_current_store(&self, user_id: Ulid, store_id: Ulid) -> AppResult<User> {
        let row = sqlx::query_as!(
            UserModel,
            r#"
            UPDATE users
            SET current_store_id = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING id, openid, nickname, avatar, phone, current_store_id, is_member, created_at, updated_at
            "#,
            user_id.to_string(),
            store_id.to_string()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| map_unique_violation(err, "手机号已绑定其他账号"))?;

        row.into_entity()
    }

    async fn find_by_phone(&self, phone: &str) -> AppResult<Option<User>> {
        let row = sqlx::query_as!(
            UserModel,
            r#"
            SELECT id, openid, nickname, avatar, phone, current_store_id, is_member, created_at, updated_at
            FROM users
            WHERE phone = $1
            "#,
            phone
        )
        .fetch_optional(&self.pool)
        .await.map_err(map_sqlx_error)?;

        match row {
            Some(model) => Ok(Some(model.into_entity()?)),
            None => Ok(None),
        }
    }

    async fn bind_phone(&self, user_id: Ulid, phone: String) -> AppResult<User> {
        let row = sqlx::query_as!(
            UserModel,
            r#"
            UPDATE users
            SET phone = $2, updated_at = NOW()
            WHERE id = $1
            RETURNING id, openid, nickname, avatar, phone, current_store_id, is_member, created_at, updated_at
            "#,
            user_id.to_string(),
            phone
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| map_unique_violation(err, "手机号已绑定其他账号"))?;

        row.into_entity()
    }
}
