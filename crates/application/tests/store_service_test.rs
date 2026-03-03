use std::collections::HashMap;
use std::sync::Arc;

use axum_application::StoreService;
use axum_application::services::store_service;
use axum_common::AppResult;
use axum_domain::store::entity::{Store, StoreStatus};
use axum_domain::store::repo::StoreRepository;
use async_trait::async_trait;
use tokio::sync::Mutex;
use ulid::Ulid;

#[derive(Default)]
struct InMemoryStoreRepo {
    inner: Mutex<HashMap<Ulid, Store>>,
}

#[async_trait]
impl StoreRepository for InMemoryStoreRepo {
    async fn list(&self) -> AppResult<Vec<Store>> {
        let guard = self.inner.lock().await;
        Ok(guard.values().cloned().collect())
    }

    async fn create(&self, store: &Store) -> AppResult<Store> {
        let mut guard = self.inner.lock().await;
        guard.insert(store.id, store.clone());
        Ok(store.clone())
    }
}

#[derive(Default)]
struct FakeLbs;

#[async_trait]
impl store_service::LbsService for FakeLbs {
    async fn distance_km(&self, _from: (f64, f64), to: (f64, f64)) -> AppResult<f64> {
        Ok(to.0)
    }
}

#[tokio::test]
async fn test_nearby_sorted_by_distance() {
    let repo: Arc<dyn StoreRepository> = Arc::new(InMemoryStoreRepo::default());
    let lbs: Arc<dyn store_service::LbsService> = Arc::new(FakeLbs::default());
    let service = StoreService::new(repo.clone(), lbs);

    let store_a = Store::new(
        "Store A".into(),
        "Addr A".into(),
        1.0,
        0.0,
        "13800000000".into(),
        "9-21".into(),
        StoreStatus::Open,
        3.0,
        0,
        2,
        2,
    )
    .unwrap();
    let store_b = Store::new(
        "Store B".into(),
        "Addr B".into(),
        2.0,
        0.0,
        "13800000001".into(),
        "9-21".into(),
        StoreStatus::Open,
        3.0,
        0,
        2,
        2,
    )
    .unwrap();

    repo.create(&store_a).await.unwrap();
    repo.create(&store_b).await.unwrap();

    let result = service.nearby(0.0, 0.0).await.unwrap();
    assert_eq!(result[0].store.id, store_a.id);
    assert_eq!(result[1].store.id, store_b.id);
}

#[tokio::test]
async fn test_delivery_fee_calc() {
    let repo: Arc<dyn StoreRepository> = Arc::new(InMemoryStoreRepo::default());
    let lbs: Arc<dyn store_service::LbsService> = Arc::new(FakeLbs::default());
    let service = StoreService::new(repo.clone(), lbs);

    let store = Store::new(
        "Store".into(),
        "Addr".into(),
        5.2,
        0.0,
        "138".into(),
        "9-21".into(),
        StoreStatus::Open,
        3.0,
        0,
        2,
        2,
    )
    .unwrap();
    repo.create(&store).await.unwrap();

    let result = service.nearby(0.0, 0.0).await.unwrap();
    let fee = result[0].delivery_fee;
    assert_eq!(fee, 6);
}
