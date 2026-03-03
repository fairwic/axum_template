//! Store service

use std::sync::Arc;

use axum_common::AppResult;
use axum_domain::store::entity::Store;
use axum_domain::store::repo::StoreRepository;
use async_trait::async_trait;

#[async_trait]
pub trait LbsService: Send + Sync {
    async fn distance_km(&self, from: (f64, f64), to: (f64, f64)) -> AppResult<f64>;
}

#[derive(Debug, Clone)]
pub struct StoreWithDistance {
    pub store: Store,
    pub distance_km: f64,
    pub deliverable: bool,
    pub delivery_fee: i32,
}

#[derive(Clone)]
pub struct StoreService {
    repo: Arc<dyn StoreRepository>,
    lbs: Arc<dyn LbsService>,
}

impl StoreService {
    pub fn new(repo: Arc<dyn StoreRepository>, lbs: Arc<dyn LbsService>) -> Self {
        Self { repo, lbs }
    }

    pub async fn nearby(&self, lat: f64, lng: f64) -> AppResult<Vec<StoreWithDistance>> {
        let stores = self.repo.list().await?;
        let mut items = Vec::with_capacity(stores.len());

        for store in stores {
            let distance_km = self
                .lbs
                .distance_km((lat, lng), (store.lat, store.lng))
                .await?;
            let delivery_fee = calc_delivery_fee(&store, distance_km);
            items.push(StoreWithDistance {
                store,
                distance_km,
                deliverable: true,
                delivery_fee,
            });
        }

        items.sort_by(|a, b| a.distance_km.partial_cmp(&b.distance_km).unwrap());
        Ok(items)
    }
}

fn calc_delivery_fee(store: &Store, distance_km: f64) -> i32 {
    if distance_km <= store.delivery_radius_km {
        return 0;
    }
    let extra = (distance_km - store.delivery_radius_km).ceil() as i32;
    store.delivery_fee_base + extra * store.delivery_fee_per_km
}
