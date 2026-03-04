use async_trait::async_trait;
use serde_json::Value;

use {
    crate::snapshot::model::{NormalizedSnapshotEvent, Platform, ProductSnapshot, ShopSnapshot},
    crate::DomainError,
};

#[async_trait]
pub trait PlatformSnapshotAdapter: Send + Sync {
    fn platform(&self) -> Platform;
    fn parse_product(&self, payload: Value) -> Result<ProductSnapshot, DomainError>;
    fn parse_shop(&self, payload: Value) -> Result<ShopSnapshot, DomainError>;
}

#[async_trait]
pub trait HotStore: Send + Sync {
    async fn save_product(&self, snapshot: &ProductSnapshot) -> Result<(), DomainError>;
    async fn save_shop(&self, snapshot: &ShopSnapshot) -> Result<(), DomainError>;
}

#[async_trait]
pub trait ColdStore: Send + Sync {
    async fn archive_product(&self, snapshot: &ProductSnapshot) -> Result<(), DomainError>;
    async fn archive_shop(&self, snapshot: &ShopSnapshot) -> Result<(), DomainError>;
}

#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(
        &self,
        topic: &str,
        event: &NormalizedSnapshotEvent,
    ) -> Result<(), DomainError>;
}
