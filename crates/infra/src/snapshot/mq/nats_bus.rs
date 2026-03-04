use async_trait::async_trait;

use axum_domain::{
    DomainError, snapshot::model::NormalizedSnapshotEvent, snapshot::ports::EventPublisher,
};

pub struct NatsEventBus {
    client: async_nats::Client,
}

impl NatsEventBus {
    pub async fn connect(url: &str) -> Result<Self, DomainError> {
        let client = async_nats::connect(url)
            .await
            .map_err(|e| DomainError::EventPublish(e.to_string()))?;
        Ok(Self { client })
    }
}

#[async_trait]
impl EventPublisher for NatsEventBus {
    async fn publish(
        &self,
        topic: &str,
        event: &NormalizedSnapshotEvent,
    ) -> Result<(), DomainError> {
        let payload =
            serde_json::to_vec(event).map_err(|e| DomainError::EventPublish(e.to_string()))?;
        self.client
            .publish(topic.to_string(), payload.into())
            .await
            .map_err(|e| DomainError::EventPublish(e.to_string()))?;
        Ok(())
    }
}
