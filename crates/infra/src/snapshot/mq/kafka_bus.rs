use std::time::Duration;

use async_trait::async_trait;
use rdkafka::{
    ClientConfig,
    producer::{FutureProducer, FutureRecord},
};

use axum_domain::{
    DomainError, snapshot::model::NormalizedSnapshotEvent, snapshot::ports::EventPublisher,
};

pub struct KafkaEventBus {
    producer: FutureProducer,
}

impl KafkaEventBus {
    pub fn connect(brokers: &str) -> Result<Self, DomainError> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .set("acks", "all")
            .set("enable.idempotence", "true")
            .create()
            .map_err(|e| DomainError::EventPublish(e.to_string()))?;

        Ok(Self { producer })
    }
}

#[async_trait]
impl EventPublisher for KafkaEventBus {
    async fn publish(
        &self,
        topic: &str,
        event: &NormalizedSnapshotEvent,
    ) -> Result<(), DomainError> {
        let payload =
            serde_json::to_vec(event).map_err(|e| DomainError::EventPublish(e.to_string()))?;
        let key = event.id.to_string();

        self.producer
            .send(
                FutureRecord::to(topic).payload(&payload).key(&key),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(e, _)| DomainError::EventPublish(e.to_string()))?;

        Ok(())
    }
}
