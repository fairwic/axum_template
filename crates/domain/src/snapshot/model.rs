use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ulid::Ulid;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Platform {
    Temu,
    Yandex,
    Other(String),
}

impl Platform {
    pub fn parse(raw: &str) -> Self {
        match raw.to_ascii_lowercase().as_str() {
            "temu" => Self::Temu,
            "yandex" => Self::Yandex,
            v => Self::Other(v.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Temu => "temu",
            Self::Yandex => "yandex",
            Self::Other(v) => v.as_str(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SnapshotKind {
    Product,
    Shop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AsyncState {
    Received,
    Normalized,
    HotPersisted,
    ColdPersisted,
    EventPublished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductSnapshot {
    pub trace_id: Ulid,
    pub platform: Platform,
    pub platform_product_id: String,
    pub platform_shop_id: String,
    pub sku: Option<String>,
    pub title: String,
    pub price_minor: i64,
    pub old_price_minor: Option<i64>,
    pub rating: Option<f64>,
    pub rating_count: Option<i64>,
    pub sales: Option<i64>,
    pub category_id: Option<String>,
    pub category_level1_id: Option<String>,
    pub category_slug: Option<String>,
    pub vendor_id: Option<String>,
    pub image_urls: Vec<String>,
    pub observed_at: DateTime<Utc>,
    pub raw_payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopSnapshot {
    pub trace_id: Ulid,
    pub platform: Platform,
    pub platform_shop_id: String,
    pub name: Option<String>,
    pub score: Option<f64>,
    pub logo: Option<String>,
    pub comment_count: Option<i64>,
    pub followers: Option<i64>,
    pub total_sales: Option<i64>,
    pub information: Option<String>,
    pub observed_at: DateTime<Utc>,
    pub raw_payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedSnapshotEvent {
    pub id: Ulid,
    pub platform: String,
    pub kind: SnapshotKind,
    pub state: AsyncState,
    pub aggregate_id: String,
    pub occurred_at: DateTime<Utc>,
    pub payload: SnapshotPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "lowercase")]
pub enum SnapshotPayload {
    Product(ProductSnapshotPayload),
    Shop(ShopSnapshotPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductSnapshotPayload {
    pub trace_id: Ulid,
    pub platform: String,
    pub platform_product_id: String,
    pub platform_shop_id: String,
    pub sku: Option<String>,
    pub title: String,
    pub price_minor: i64,
    pub old_price_minor: Option<i64>,
    pub rating: Option<f64>,
    pub rating_count: Option<i64>,
    pub sales: Option<i64>,
    pub category_id: Option<String>,
    pub category_level1_id: Option<String>,
    pub category_slug: Option<String>,
    pub vendor_id: Option<String>,
    pub image_urls: Vec<String>,
    pub observed_at: DateTime<Utc>,
}

impl ProductSnapshotPayload {
    pub fn from_snapshot(snapshot: &ProductSnapshot) -> Self {
        Self {
            trace_id: snapshot.trace_id,
            platform: snapshot.platform.as_str().to_string(),
            platform_product_id: snapshot.platform_product_id.clone(),
            platform_shop_id: snapshot.platform_shop_id.clone(),
            sku: snapshot.sku.clone(),
            title: snapshot.title.clone(),
            price_minor: snapshot.price_minor,
            old_price_minor: snapshot.old_price_minor,
            rating: snapshot.rating,
            rating_count: snapshot.rating_count,
            sales: snapshot.sales,
            category_id: snapshot.category_id.clone(),
            category_level1_id: snapshot.category_level1_id.clone(),
            category_slug: snapshot.category_slug.clone(),
            vendor_id: snapshot.vendor_id.clone(),
            image_urls: snapshot.image_urls.clone(),
            observed_at: snapshot.observed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopSnapshotPayload {
    pub trace_id: Ulid,
    pub platform: String,
    pub platform_shop_id: String,
    pub name: Option<String>,
    pub score: Option<f64>,
    pub logo: Option<String>,
    pub comment_count: Option<i64>,
    pub followers: Option<i64>,
    pub total_sales: Option<i64>,
    pub information: Option<String>,
    pub observed_at: DateTime<Utc>,
}

impl ShopSnapshotPayload {
    pub fn from_snapshot(snapshot: &ShopSnapshot) -> Self {
        Self {
            trace_id: snapshot.trace_id,
            platform: snapshot.platform.as_str().to_string(),
            platform_shop_id: snapshot.platform_shop_id.clone(),
            name: snapshot.name.clone(),
            score: snapshot.score,
            logo: snapshot.logo.clone(),
            comment_count: snapshot.comment_count,
            followers: snapshot.followers,
            total_sales: snapshot.total_sales,
            information: snapshot.information.clone(),
            observed_at: snapshot.observed_at,
        }
    }
}
