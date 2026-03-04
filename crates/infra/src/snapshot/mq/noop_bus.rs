//! Noop 事件发布器，用于开发/测试环境

use async_trait::async_trait;
use axum_domain::{
    snapshot::model::NormalizedSnapshotEvent, snapshot::ports::EventPublisher, DomainError,
};

/// 仅打 log，不真实发送消息
#[derive(Clone, Default)]
pub struct NoopEventPublisher;

#[async_trait]
impl EventPublisher for NoopEventPublisher {
    async fn publish(
        &self,
        topic: &str,
        event: &NormalizedSnapshotEvent,
    ) -> Result<(), DomainError> {
        tracing::debug!(topic, event_id = %event.id, "noop publisher: skip publish");
        Ok(())
    }
}
