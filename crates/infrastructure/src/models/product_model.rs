//! Product model for persistence

use axum_core_kernel::AppError;
use axum_domain::product::entity::{Product, ProductStatus};
use chrono::{DateTime, Utc};
use serde_json::Value;
use ulid::Ulid;

#[derive(Debug, sqlx::FromRow)]
pub struct ProductModel {
    pub id: String,
    pub store_id: String,
    pub category_id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub cover_image: String,
    pub images: Value,
    pub price: i32,
    pub original_price: Option<i32>,
    pub stock: i32,
    pub status: String,
    pub tags: Value,
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
            images: Value::Array(product.images.iter().cloned().map(Value::String).collect()),
            price: product.price,
            original_price: product.original_price,
            stock: product.stock,
            status: match product.status {
                ProductStatus::On => "ON".to_string(),
                ProductStatus::Off => "OFF".to_string(),
            },
            tags: Value::Array(product.tags.iter().cloned().map(Value::String).collect()),
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
        let images: Vec<String> = serde_json::from_value(self.images)
            .map_err(|_| AppError::Internal("invalid images json".into()))?;
        let tags: Vec<String> = serde_json::from_value(self.tags)
            .map_err(|_| AppError::Internal("invalid tags json".into()))?;
        Ok(Product {
            id,
            store_id,
            category_id,
            title: self.title,
            subtitle: self.subtitle,
            cover_image: self.cover_image,
            images,
            price: self.price,
            original_price: self.original_price,
            stock: self.stock,
            status,
            tags,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
