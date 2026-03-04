use chrono::Utc;
use serde_json::Value;
use ulid::Ulid;

use super::common::{pick_f64, pick_first_array_item_string, pick_i64, pick_images, pick_string};
use axum_domain::{
    error::DomainError,
    snapshot::model::{Platform, ProductSnapshot, ShopSnapshot},
    snapshot::ports::PlatformSnapshotAdapter,
};

#[derive(Debug, Default)]
pub struct TemuAdapter;

impl TemuAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl PlatformSnapshotAdapter for TemuAdapter {
    fn platform(&self) -> Platform {
        Platform::Temu
    }

    fn parse_product(&self, payload: Value) -> Result<ProductSnapshot, DomainError> {
        // 新字段优先，旧字段别名兼容
        let platform_product_id =
            pick_string(&payload, &["sku_id", "goods_id", "id", "product_id"]).ok_or_else(
                || DomainError::InvalidPayload("temu product_id missing".to_string()),
            )?;
        let platform_shop_id = pick_string(&payload, &["business_id", "mall_id", "shop_id"])
            .ok_or_else(|| DomainError::InvalidPayload("temu shop_id missing".to_string()))?;
        let title = pick_string(&payload, &["title", "name"])
            .ok_or_else(|| DomainError::InvalidPayload("temu title missing".to_string()))?;

        Ok(ProductSnapshot {
            trace_id: Ulid::new(),
            platform: Platform::Temu,
            platform_product_id,
            platform_shop_id,
            sku: pick_string(&payload, &["market_sku", "sku_id"]),
            title,
            price_minor: pick_i64(&payload, &["price"]).unwrap_or_default(),
            old_price_minor: pick_i64(&payload, &["old_price", "market_price"]),
            rating: pick_f64(&payload, &["rating", "score"]),
            rating_count: pick_i64(&payload, &["rating_count", "comment_num_tips"]),
            sales: pick_i64(&payload, &["sales", "sales_num"]),
            category_id: pick_string(&payload, &["category_hid", "category_id", "opt_id"]),
            category_level1_id: pick_first_array_item_string(&payload, "category_hids"),
            category_slug: pick_string(&payload, &["category_slug"]),
            vendor_id: pick_string(&payload, &["vendor_id"]),
            image_urls: pick_images(&payload),
            observed_at: Utc::now(),
            raw_payload: payload,
        })
    }

    fn parse_shop(&self, payload: Value) -> Result<ShopSnapshot, DomainError> {
        let platform_shop_id = pick_string(&payload, &["mall_id", "shop_id", "business_id", "id"])
            .ok_or_else(|| DomainError::InvalidPayload("temu shop_id missing".to_string()))?;

        Ok(ShopSnapshot {
            trace_id: Ulid::new(),
            platform: Platform::Temu,
            platform_shop_id,
            name: pick_string(&payload, &["name", "shop_name"]),
            score: pick_f64(&payload, &["score", "rating"]),
            logo: pick_string(&payload, &["logo", "shop_logo"]),
            comment_count: pick_i64(&payload, &["comment_count"]),
            followers: pick_i64(&payload, &["followers"]),
            total_sales: pick_i64(&payload, &["total_sales"]),
            information: pick_string(&payload, &["information"]),
            observed_at: Utc::now(),
            raw_payload: payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_product_supports_new_yandex_like_keys_for_backward_compat() {
        let adapter = TemuAdapter::new();
        let payload = json!({
            "sku_id": 103790630154_u64,
            "business_id": "100008355",
            "title": "new format product",
            "price": 112203,
            "old_price": 1122030000000_i64,
            "category_hid": 1003093,
            "category_hids": [6179129],
            "category_slug": "mebel",
            "vendor_id": 53169608,
            "images": ["https://img/1"]
        });

        let snapshot = adapter.parse_product(payload).expect("parse product");
        assert_eq!(snapshot.platform_product_id, "103790630154");
        assert_eq!(snapshot.platform_shop_id, "100008355");
        assert_eq!(snapshot.category_id.as_deref(), Some("1003093"));
        assert_eq!(snapshot.category_level1_id.as_deref(), Some("6179129"));
        assert_eq!(snapshot.category_slug.as_deref(), Some("mebel"));
        assert_eq!(snapshot.vendor_id.as_deref(), Some("53169608"));
        assert_eq!(snapshot.image_urls.len(), 1);
    }

    #[test]
    fn parse_product_supports_legacy_temu_keys() {
        let adapter = TemuAdapter::new();
        let payload = json!({
            "goods_id": "601099536052370",
            "mall_id": "10021691104",
            "title": "legacy temu product",
            "price": 999,
            "sales_num": 11,
            "comment_num_tips": 3
        });

        let snapshot = adapter.parse_product(payload).expect("parse product");
        assert_eq!(snapshot.platform_product_id, "601099536052370");
        assert_eq!(snapshot.platform_shop_id, "10021691104");
        assert_eq!(snapshot.sales, Some(11));
        assert_eq!(snapshot.rating_count, Some(3));
    }
}
