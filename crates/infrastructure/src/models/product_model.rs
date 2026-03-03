//! Product model for persistence

use axum_common::AppError;
use axum_domain::product::entity::{Product, ProductStatus};
use chrono::{DateTime, Utc};
use sqlx::types::Json;
use ulid::Ulid;

#[derive(Debug, sqlx::FromRow)]
pub struct ProductModel {
    pub id: String,
    pub store_id: String,
    pub category_id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub cover_image: String,
    pub images: Json<Vec<String>>,
    pub price: i32,
    pub original_price: Option<i32>,
    pub stock: i32,
    pub status: String,
    pub tags: Json<Vec<String>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ProductModel {
    pub fn from_entity(product: &Product) -> Self {
        Self {
            id: product.id.to_string(),
            store_id: product.store_id.to_string(),
            category_id: product.category_id.to_string(),
            title: product.title.clone(),
            subtitle: product.subtitle.clone(),
            cover_image: product.cover_image.clone(),
            images: Json(product.images.clone()),
            price: product.price,
            original_price: product.original_price,
            stock: product.stock,
            status: match product.status {
                ProductStatus::On => "ON".to_string(),
                ProductStatus::Off => "OFF".to_string(),
            },
            tags: Json(product.tags.clone()),
            created_at: product.created_at,
            updated_at: product.updated_at,
        }
    }

    pub fn into_entity(self) -> Result<Product, AppError> {
        let id = Ulid::from_string(&self.id)
            .map_err(|_| AppError::Internal("invalid ulid in database".into()))?;
        let store_id = Ulid::from_string(&self.store_id)
            .map_err(|_| AppError::Internal("invalid store_id in database".into()))?;
        let category_id = Ulid::from_string(&self.category_id)
            .map_err(|_| AppError::Internal("invalid category_id in database".into()))?;
        let status = match self.status.as_str() {
            "ON" => ProductStatus::On,
            "OFF" => ProductStatus::Off,
            _ => return Err(AppError::Internal("invalid product status".into())),
        };
        Ok(Product {
            id,
            store_id,
            category_id,
            title: self.title,
            subtitle: self.subtitle,
            cover_image: self.cover_image,
            images: self.images.0,
            price: self.price,
            original_price: self.original_price,
            stock: self.stock,
            status,
            tags: self.tags.0,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
