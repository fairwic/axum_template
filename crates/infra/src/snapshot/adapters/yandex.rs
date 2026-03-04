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
pub struct YandexAdapter;

impl YandexAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl PlatformSnapshotAdapter for YandexAdapter {
    fn platform(&self) -> Platform {
        Platform::Yandex
    }

    fn parse_product(&self, payload: Value) -> Result<ProductSnapshot, DomainError> {
        // 新字段优先，旧字段别名兼容
        let platform_product_id =
            pick_string(&payload, &["sku_id", "goods_id", "id", "product_id"]).ok_or_else(
                || DomainError::InvalidPayload("yandex product_id missing".to_string()),
            )?;
        let platform_shop_id = pick_string(&payload, &["business_id", "mall_id", "shop_id"])
            .ok_or_else(|| {
                DomainError::InvalidPayload("yandex business_id/shop_id missing".to_string())
            })?;
        let title = pick_string(&payload, &["title", "name"])
            .ok_or_else(|| DomainError::InvalidPayload("yandex title missing".to_string()))?;
        let price_minor = pick_i64(&payload, &["price"])
            .ok_or_else(|| DomainError::InvalidPayload("yandex price missing".to_string()))?;

        Ok(ProductSnapshot {
            trace_id: Ulid::new(),
            platform: Platform::Yandex,
            platform_product_id,
            platform_shop_id,
            sku: pick_string(&payload, &["market_sku", "sku"]),
            title,
            price_minor,
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
        let platform_shop_id = pick_string(&payload, &["mall_id", "business_id", "shop_id", "id"])
            .ok_or_else(|| DomainError::InvalidPayload("yandex shop_id missing".to_string()))?;

        Ok(ShopSnapshot {
            trace_id: Ulid::new(),
            platform: Platform::Yandex,
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
    fn parse_product_supports_new_yandex_payload() {
        let adapter = YandexAdapter::new();
        let payload = json!({
          "title": "Парящая кровать",
          "price": 112203,
          "old_price": 1122030000000_i64,
          "market_sku": "4814224366",
          "business_id": "100008355",
          "vendor_id": 53169608,
          "sku_id": 103790630154_u64,
          "category_hid": 1003093,
          "category_hids": [6179129],
          "category_slug": "mebel",
          "rating": 4.7,
          "rating_count": 3,
          "sales": 7,
          "images": ["https://avatars.mds.yandex.net/get-mpic/1/orig"]
        });

        let snapshot = adapter.parse_product(payload).expect("parse product");
        assert_eq!(snapshot.platform_product_id, "103790630154");
        assert_eq!(snapshot.platform_shop_id, "100008355");
        assert_eq!(snapshot.category_id.as_deref(), Some("1003093"));
        assert_eq!(snapshot.category_level1_id.as_deref(), Some("6179129"));
        assert_eq!(snapshot.category_slug.as_deref(), Some("mebel"));
        assert_eq!(snapshot.vendor_id.as_deref(), Some("53169608"));
        assert_eq!(snapshot.price_minor, 112203);
    }

    #[test]
    fn parse_shop_supports_backward_compatible_keys() {
        let adapter = YandexAdapter::new();
        let payload = json!({
            "business_id": "100008355",
            "name": "shop name",
            "score": 4.8,
            "logo": "https://logo",
            "followers": 11,
            "comment_count": 3,
            "total_sales": 77,
            "information": "主体"
        });

        let snapshot = adapter.parse_shop(payload).expect("parse shop");
        assert_eq!(snapshot.platform_shop_id, "100008355");
        assert_eq!(snapshot.followers, Some(11));
        assert_eq!(snapshot.comment_count, Some(3));
    }
}
