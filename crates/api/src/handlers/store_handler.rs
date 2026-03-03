//! Store handlers

use axum::extract::{Query, State};
use axum_common::{ApiResponse, AppResult};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use axum_domain::store::entity::{Store, StoreStatus};

use crate::state::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct NearbyQuery {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StoreNearbyResponse {
    pub id: String,
    pub name: String,
    pub address: String,
    pub lat: f64,
    pub lng: f64,
    pub phone: String,
    pub business_hours: String,
    pub status: String,
    pub distance_km: f64,
    pub deliverable: bool,
    pub delivery_fee: i32,
}

fn status_to_string(status: &StoreStatus) -> String {
    match status {
        StoreStatus::Open => "OPEN".to_string(),
        StoreStatus::Closed => "CLOSED".to_string(),
    }
}

fn to_response(
    store: Store,
    distance_km: f64,
    deliverable: bool,
    delivery_fee: i32,
) -> StoreNearbyResponse {
    StoreNearbyResponse {
        id: store.id.to_string(),
        name: store.name,
        address: store.address,
        lat: store.lat,
        lng: store.lng,
        phone: store.phone,
        business_hours: store.business_hours,
        status: status_to_string(&store.status),
        distance_km,
        deliverable,
        delivery_fee,
    }
}

#[utoipa::path(
    get,
    path = "/stores/nearby",
    params(("lat" = f64, Query, description = "纬度"), ("lng" = f64, Query, description = "经度")),
    responses((status = 200, description = "Nearby stores", body = ApiResponse<Vec<StoreNearbyResponse>>)),
    tag = "Store"
)]
pub async fn nearby_stores(
    State(state): State<AppState>,
    Query(query): Query<NearbyQuery>,
) -> AppResult<ApiResponse<Vec<StoreNearbyResponse>>> {
    let items = state.store_service.nearby(query.lat, query.lng).await?;
    let data = items
        .into_iter()
        .map(|item| {
            to_response(
                item.store,
                item.distance_km,
                item.deliverable,
                item.delivery_fee,
            )
        })
        .collect();
    Ok(ApiResponse::success(data))
}
