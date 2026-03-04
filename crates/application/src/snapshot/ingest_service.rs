use std::{collections::HashMap, sync::Arc};

use chrono::Utc;
use serde_json::Value;
use ulid::Ulid;

use axum_domain::{
    snapshot::model::{
        AsyncState, NormalizedSnapshotEvent, Platform, ProductSnapshotPayload, ShopSnapshotPayload,
        SnapshotKind, SnapshotPayload,
    },
    snapshot::ports::{EventPublisher, HotStore, PlatformSnapshotAdapter},
    DomainError,
};

pub struct IngestService {
    hot_store: Arc<dyn HotStore>,
    publisher: Arc<dyn EventPublisher>,
    adapters: HashMap<Platform, Arc<dyn PlatformSnapshotAdapter>>,
}

impl IngestService {
    pub fn new(
        hot_store: Arc<dyn HotStore>,
        publisher: Arc<dyn EventPublisher>,
        adapters: Vec<Arc<dyn PlatformSnapshotAdapter>>,
    ) -> Self {
        let mut map = HashMap::with_capacity(adapters.len());
        for adapter in adapters {
            map.insert(adapter.platform(), adapter);
        }
        Self {
            hot_store,
            publisher,
            adapters: map,
        }
    }

    pub async fn ingest_product(
        &self,
        platform: Platform,
        payload: Value,
    ) -> Result<Ulid, DomainError> {
        let adapter = self
            .adapters
            .get(&platform)
            .ok_or_else(|| DomainError::AdapterNotFound(platform.as_str().to_string()))?;

        let snapshot = adapter.parse_product(payload)?;
        self.hot_store.save_product(&snapshot).await?;

        let event = NormalizedSnapshotEvent {
            id: Ulid::new(),
            platform: platform.as_str().to_string(),
            kind: SnapshotKind::Product,
            state: AsyncState::EventPublished,
            aggregate_id: snapshot.platform_product_id.clone(),
            occurred_at: Utc::now(),
            payload: SnapshotPayload::Product(ProductSnapshotPayload::from_snapshot(&snapshot)),
        };

        self.publisher
            .publish(&product_topic(platform.as_str()), &event)
            .await?;

        Ok(snapshot.trace_id)
    }

    pub async fn ingest_shop(
        &self,
        platform: Platform,
        payload: Value,
    ) -> Result<Ulid, DomainError> {
        let adapter = self
            .adapters
            .get(&platform)
            .ok_or_else(|| DomainError::AdapterNotFound(platform.as_str().to_string()))?;

        let snapshot = adapter.parse_shop(payload)?;
        self.hot_store.save_shop(&snapshot).await?;

        let event = NormalizedSnapshotEvent {
            id: Ulid::new(),
            platform: platform.as_str().to_string(),
            kind: SnapshotKind::Shop,
            state: AsyncState::EventPublished,
            aggregate_id: snapshot.platform_shop_id.clone(),
            occurred_at: Utc::now(),
            payload: SnapshotPayload::Shop(ShopSnapshotPayload::from_snapshot(&snapshot)),
        };

        self.publisher
            .publish(&shop_topic(platform.as_str()), &event)
            .await?;

        Ok(snapshot.trace_id)
    }
}

fn product_topic(platform: &str) -> String {
    format!("snapshot.{platform}.product.normalized")
}

fn shop_topic(platform: &str) -> String {
    format!("snapshot.{platform}.shop.normalized")
}
