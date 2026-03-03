//! Product entity

use crate::DomainError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProductStatus {
    On,
    Off,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Product {
    pub id: Ulid,
    pub store_id: Ulid,
    pub category_id: Ulid,
    pub title: String,
    pub subtitle: Option<String>,
    pub cover_image: String,
    pub images: Vec<String>,
    pub price: i32,
    pub original_price: Option<i32>,
    pub stock: i32,
    pub status: ProductStatus,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Product {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        store_id: Ulid,
        category_id: Ulid,
        title: String,
        subtitle: Option<String>,
        cover_image: String,
        images: Vec<String>,
        price: i32,
        original_price: Option<i32>,
        stock: i32,
        status: ProductStatus,
        tags: Vec<String>,
    ) -> Result<Self, DomainError> {
        if title.trim().is_empty() {
            return Err(DomainError::Validation("title is required".into()));
        }
        if price < 0 || stock < 0 {
            return Err(DomainError::Validation("price/stock invalid".into()));
        }
        let now = Utc::now();
        Ok(Self {
            id: Ulid::new(),
            store_id,
            category_id,
            title,
            subtitle,
            cover_image,
            images,
            price,
            original_price,
            stock,
            status,
            tags,
            created_at: now,
            updated_at: now,
        })
    }
}
