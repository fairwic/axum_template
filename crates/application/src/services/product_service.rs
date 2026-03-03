//! Product service

use std::sync::Arc;

use axum_common::{AppError, AppResult, PagedResponse};
use axum_domain::product::entity::Product;
use axum_domain::product::repo::ProductRepository;
use ulid::Ulid;

#[derive(Clone)]
pub struct ProductService {
    repo: Arc<dyn ProductRepository>,
}

impl ProductService {
    pub fn new(repo: Arc<dyn ProductRepository>) -> Self {
        Self { repo }
    }

    pub async fn list_by_category(
        &self,
        store_id: Ulid,
        category_id: Ulid,
        page: i64,
        page_size: i64,
    ) -> AppResult<PagedResponse<Product>> {
        let (items, total) = self
            .repo
            .list_by_category(store_id, category_id, page, page_size)
            .await?;
        Ok(PagedResponse::new(items, total, page, page_size))
    }

    pub async fn search(
        &self,
        store_id: Ulid,
        keyword: &str,
        page: i64,
        page_size: i64,
    ) -> AppResult<PagedResponse<Product>> {
        let (items, total) = self.repo.search(store_id, keyword, page, page_size).await?;
        Ok(PagedResponse::new(items, total, page, page_size))
    }

    pub async fn get_by_id(&self, store_id: Ulid, product_id: Ulid) -> AppResult<Product> {
        self.repo
            .find_by_id(store_id, product_id)
            .await?
            .ok_or_else(|| AppError::NotFound("product not found".into()))
    }
}
