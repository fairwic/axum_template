//! Category service

use std::sync::Arc;

use axum_common::{AppError, AppResult};
use axum_domain::category::entity::Category;
use axum_domain::category::repo::CategoryRepository;
use chrono::Utc;
use ulid::Ulid;

use crate::dtos::category_dto::{CreateCategoryInput, UpdateCategoryInput};

#[derive(Clone)]
pub struct CategoryService {
    repo: Arc<dyn CategoryRepository>,
}

impl CategoryService {
    pub fn new(repo: Arc<dyn CategoryRepository>) -> Self {
        Self { repo }
    }

    pub async fn list_by_store(&self, store_id: Ulid) -> AppResult<Vec<Category>> {
        self.repo.list_by_store(store_id).await
    }

    pub async fn admin_create(&self, input: CreateCategoryInput) -> AppResult<Category> {
        let category = Category::new(input.store_id, input.name, input.sort_order, input.status)?;
        self.repo.create(&category).await
    }

    pub async fn admin_update(
        &self,
        category_id: Ulid,
        input: UpdateCategoryInput,
    ) -> AppResult<Category> {
        let existing = self
            .repo
            .find_by_id(category_id)
            .await?
            .ok_or_else(|| AppError::NotFound("category not found".into()))?;
        let mut category = Category::new(
            existing.store_id,
            input.name,
            input.sort_order,
            input.status,
        )?;
        category.id = existing.id;
        category.created_at = existing.created_at;
        category.updated_at = Utc::now();
        self.repo.update(&category).await
    }
}
