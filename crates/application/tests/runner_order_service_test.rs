use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum_application::{CreateRunnerOrderInput, RunnerOrderService};
use axum_common::AppResult;
use axum_domain::order::entity::PayStatus;
use axum_domain::runner_order::entity::{RunnerOrder, RunnerOrderStatus};
use axum_domain::runner_order::repo::RunnerOrderRepository;
use tokio::sync::Mutex;
use ulid::Ulid;

#[derive(Default)]
struct InMemoryRunnerOrderRepo {
    inner: Mutex<HashMap<Ulid, RunnerOrder>>,
}

#[async_trait]
impl RunnerOrderRepository for InMemoryRunnerOrderRepo {
    async fn create(&self, order: &RunnerOrder) -> AppResult<RunnerOrder> {
        let mut guard = self.inner.lock().await;
        guard.insert(order.id, order.clone());
        Ok(order.clone())
    }

    async fn update(&self, order: &RunnerOrder) -> AppResult<RunnerOrder> {
        let mut guard = self.inner.lock().await;
        guard.insert(order.id, order.clone());
        Ok(order.clone())
    }

    async fn find_by_id(&self, order_id: Ulid) -> AppResult<Option<RunnerOrder>> {
        let guard = self.inner.lock().await;
        Ok(guard.get(&order_id).cloned())
    }

    async fn list_by_user(&self, user_id: Ulid) -> AppResult<Vec<RunnerOrder>> {
        let guard = self.inner.lock().await;
        Ok(guard
            .values()
            .filter(|item| item.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn list_by_store(&self, store_id: Ulid) -> AppResult<Vec<RunnerOrder>> {
        let guard = self.inner.lock().await;
        Ok(guard
            .values()
            .filter(|item| item.store_id == store_id)
            .cloned()
            .collect())
    }
}

#[tokio::test]
async fn test_runner_order_create_pay_cancel() {
    let repo: Arc<dyn RunnerOrderRepository> = Arc::new(InMemoryRunnerOrderRepo::default());
    let service = RunnerOrderService::new(repo);

    let user_id = Ulid::new();
    let store_id = Ulid::new();

    let order = service
        .create(CreateRunnerOrderInput {
            user_id,
            store_id,
            express_company: "顺丰".into(),
            pickup_code: "12-3-4567".into(),
            delivery_address: "A区101".into(),
            receiver_name: "张三".into(),
            receiver_phone: "13800000000".into(),
            remark: None,
            distance_km: Some(4.0),
        })
        .await
        .unwrap();
    assert_eq!(order.status, RunnerOrderStatus::PendingPay);
    assert_eq!(order.service_fee, 300);

    let paid = service.pay(user_id, order.id).await.unwrap();
    assert_eq!(paid.status, RunnerOrderStatus::PendingAccept);

    let canceled = service.cancel(user_id, order.id, None).await.unwrap();
    assert_eq!(canceled.status, RunnerOrderStatus::Canceled);
    assert_eq!(canceled.pay_status, PayStatus::Refunded);
}

#[tokio::test]
async fn test_runner_order_admin_state_flow() {
    let repo: Arc<dyn RunnerOrderRepository> = Arc::new(InMemoryRunnerOrderRepo::default());
    let service = RunnerOrderService::new(repo);

    let user_id = Ulid::new();
    let store_id = Ulid::new();

    let order = service
        .create(CreateRunnerOrderInput {
            user_id,
            store_id,
            express_company: "顺丰".into(),
            pickup_code: "12-3-4567".into(),
            delivery_address: "A区101".into(),
            receiver_name: "张三".into(),
            receiver_phone: "13800000000".into(),
            remark: None,
            distance_km: Some(2.0),
        })
        .await
        .unwrap();

    let paid = service.pay(user_id, order.id).await.unwrap();
    let accepted = service.admin_accept(paid.id).await.unwrap();
    assert_eq!(accepted.status, RunnerOrderStatus::Processing);

    let delivered = service.admin_delivered(order.id).await.unwrap();
    assert_eq!(delivered.status, RunnerOrderStatus::Delivered);

    let completed = service.admin_complete(order.id).await.unwrap();
    assert_eq!(completed.status, RunnerOrderStatus::Completed);
}
