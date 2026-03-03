//! Runner order service

use std::sync::Arc;

use axum_common::{AppError, AppResult};
use axum_domain::runner_order::entity::RunnerOrder;
use axum_domain::runner_order::repo::RunnerOrderRepository;
use ulid::Ulid;

const RUNNER_BASE_FEE: i32 = 200;
const RUNNER_SURCHARGE_PER_EXTRA_KM: i32 = 100;
const RUNNER_FREE_DISTANCE_KM: f64 = 3.0;

#[derive(Debug, Clone)]
pub struct CreateRunnerOrderInput {
    pub user_id: Ulid,
    pub store_id: Ulid,
    pub express_company: String,
    pub pickup_code: String,
    pub delivery_address: String,
    pub receiver_name: String,
    pub receiver_phone: String,
    pub remark: Option<String>,
    pub distance_km: Option<f64>,
}

#[derive(Clone)]
pub struct RunnerOrderService {
    repo: Arc<dyn RunnerOrderRepository>,
}

impl RunnerOrderService {
    pub fn new(repo: Arc<dyn RunnerOrderRepository>) -> Self {
        Self { repo }
    }

    pub async fn create(&self, input: CreateRunnerOrderInput) -> AppResult<RunnerOrder> {
        let fee = calc_service_fee(input.distance_km);
        let order = RunnerOrder::new(
            input.user_id,
            input.store_id,
            input.express_company,
            input.pickup_code,
            input.delivery_address,
            input.receiver_name,
            input.receiver_phone,
            input.remark,
            fee,
            input.distance_km,
        )?;
        self.repo.create(&order).await
    }

    pub async fn pay(&self, user_id: Ulid, order_id: Ulid) -> AppResult<RunnerOrder> {
        let mut order = self.must_get_for_user(user_id, order_id).await?;
        order.mark_paid()?;
        self.repo.update(&order).await
    }

    pub async fn cancel(
        &self,
        user_id: Ulid,
        order_id: Ulid,
        reason: Option<String>,
    ) -> AppResult<RunnerOrder> {
        let mut order = self.must_get_for_user(user_id, order_id).await?;
        order.cancel(reason)?;
        self.repo.update(&order).await
    }

    pub async fn list_by_user(&self, user_id: Ulid) -> AppResult<Vec<RunnerOrder>> {
        self.repo.list_by_user(user_id).await
    }

    pub async fn get_by_user(&self, user_id: Ulid, order_id: Ulid) -> AppResult<RunnerOrder> {
        self.must_get_for_user(user_id, order_id).await
    }

    pub async fn admin_list_by_store(&self, store_id: Ulid) -> AppResult<Vec<RunnerOrder>> {
        self.repo.list_by_store(store_id).await
    }

    pub async fn admin_accept(&self, order_id: Ulid) -> AppResult<RunnerOrder> {
        let mut order = self.must_get(order_id).await?;
        order.admin_accept()?;
        self.repo.update(&order).await
    }

    pub async fn admin_delivered(&self, order_id: Ulid) -> AppResult<RunnerOrder> {
        let mut order = self.must_get(order_id).await?;
        order.admin_delivered()?;
        self.repo.update(&order).await
    }

    pub async fn admin_complete(&self, order_id: Ulid) -> AppResult<RunnerOrder> {
        let mut order = self.must_get(order_id).await?;
        order.admin_complete()?;
        self.repo.update(&order).await
    }

    async fn must_get(&self, order_id: Ulid) -> AppResult<RunnerOrder> {
        self.repo
            .find_by_id(order_id)
            .await?
            .ok_or_else(|| AppError::NotFound("runner order not found".into()))
    }

    async fn must_get_for_user(&self, user_id: Ulid, order_id: Ulid) -> AppResult<RunnerOrder> {
        let order = self.must_get(order_id).await?;
        if order.user_id != user_id {
            return Err(AppError::Forbidden);
        }
        Ok(order)
    }
}

fn calc_service_fee(distance_km: Option<f64>) -> i32 {
    let distance_km = distance_km.unwrap_or(0.0);
    if distance_km <= RUNNER_FREE_DISTANCE_KM {
        return RUNNER_BASE_FEE;
    }
    let extra_km = (distance_km - RUNNER_FREE_DISTANCE_KM).ceil() as i32;
    RUNNER_BASE_FEE + extra_km * RUNNER_SURCHARGE_PER_EXTRA_KM
}
