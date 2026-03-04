//! Product service

use std::sync::Arc;

use axum_core_kernel::{AppError, AppResult};
use axum_domain::product::entity::Product;
use axum_domain::product::repo::ProductRepository;
use chrono::Utc;
use ulid::Ulid;

use crate::dtos::page_dto::PageResult;
use crate::dtos::product_dto::{CreateProductInput, UpdateProductInput};

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
    ) -> AppResult<PageResult<Product>> {
        let (items, total) = self
            .repo
            .list_by_category(store_id, category_id, page, page_size)
            .await?;
        Ok(PageResult::new(items, total, page, page_size))
    }

    pub async fn search(
        &self,
        store_id: Ulid,
        keyword: &str,
        page: i64,
        page_size: i64,
    ) -> AppResult<PageResult<Product>> {
        let (items, total) = self.repo.search(store_id, keyword, page, page_size).await?;
        Ok(PageResult::new(items, total, page, page_size))
    }

    pub async fn get_by_id(&self, store_id: Ulid, product_id: Ulid) -> AppResult<Product> {
        self.repo
            .find_by_id(store_id, product_id)
            .await?
            .ok_or_else(|| AppError::NotFound("product not found".into()))
    }

    pub async fn admin_create(&self, input: CreateProductInput) -> AppResult<Product> {
        let product = Product::new(
            input.store_id,
            input.category_id,
            input.title,
            input.subtitle,
            input.cover_image,
            input.images,
            input.price,
            input.original_price,
            input.stock,
            input.status,
            input.tags,
        )?;
        self.repo.create(&product).await
    }

    pub async fn admin_update(
        &self,
        product_id: Ulid,
        input: UpdateProductInput,
    ) -> AppResult<Product> {
        let existing = self
            .repo
            .find_by_id(input.store_id, product_id)
            .await?
            .ok_or_else(|| AppError::NotFound("product not found".into()))?;
        let mut product = Product::new(
            input.store_id,
            input.category_id,
            input.title,
            input.subtitle,
            input.cover_image,
            input.images,
            input.price,
            input.original_price,
            input.stock,
            input.status,
            input.tags,
        )?;
        product.id = existing.id;
        product.created_at = existing.created_at;
        product.updated_at = Utc::now();
        self.repo.update(&product).await
    }
}
