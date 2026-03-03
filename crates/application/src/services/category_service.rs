//! Category service

use std::sync::Arc;

use axum_common::AppResult;
use axum_domain::category::entity::Category;
use axum_domain::category::repo::CategoryRepository;
use ulid::Ulid;

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
}
