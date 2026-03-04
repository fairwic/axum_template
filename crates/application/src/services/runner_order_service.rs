//! Runner order service

use std::sync::Arc;

use crate::dtos::runner_order_dto::CreateRunnerOrderInput;
use axum_core_kernel::{AppError, AppResult};
use axum_domain::runner_order::entity::RunnerOrder;
use axum_domain::runner_order::repo::RunnerOrderRepository;
use axum_domain::store::repo::StoreRepository;
use axum_domain::transaction::TransactionManager;
use chrono::{Duration, Utc};
use ulid::Ulid;

const RUNNER_BASE_FEE: i32 = 200;
const RUNNER_SURCHARGE_PER_EXTRA_KM: i32 = 100;
const RUNNER_FREE_DISTANCE_KM: f64 = 3.0;

#[derive(Clone)]
pub struct RunnerOrderService {
    repo: Arc<dyn RunnerOrderRepository>,
    store_repo: Arc<dyn StoreRepository>,
    tx_manager: Option<Arc<dyn TransactionManager>>,
}

impl RunnerOrderService {
    pub fn new(repo: Arc<dyn RunnerOrderRepository>, store_repo: Arc<dyn StoreRepository>) -> Self {
        Self {
            repo,
            store_repo,
            tx_manager: None,
        }
    }

    pub fn with_transaction_manager(mut self, tx_manager: Arc<dyn TransactionManager>) -> Self {
        self.tx_manager = Some(tx_manager);
        self
    }

    async fn create_in_transaction(
        &self,
        tx_manager: &Arc<dyn TransactionManager>,
        order: &RunnerOrder,
    ) -> AppResult<RunnerOrder> {
        let mut uow = tx_manager.begin_order_uow().await?;
        let saved = match uow.create_runner_order(order).await {
            Ok(saved) => saved,
            Err(err) => {
                let _ = uow.rollback().await;
                return Err(err);
            }
        };
        uow.commit().await?;
        Ok(saved)
    }

    async fn update_in_transaction(
        &self,
        tx_manager: &Arc<dyn TransactionManager>,
        order: &RunnerOrder,
    ) -> AppResult<RunnerOrder> {
        let mut uow = tx_manager.begin_order_uow().await?;
        let updated = match uow.update_runner_order(order).await {
            Ok(updated) => updated,
            Err(err) => {
                let _ = uow.rollback().await;
                return Err(err);
            }
        };
        uow.commit().await?;
        Ok(updated)
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

        if let Some(tx_manager) = &self.tx_manager {
            return self.create_in_transaction(tx_manager, &order).await;
        }

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
        cancel_timeout_secs: u64,
    ) -> AppResult<RunnerOrder> {
        let mut order = self.must_get_for_user(user_id, order_id).await?;
        ensure_cancel_within_window(order.created_at, order.pay_time, cancel_timeout_secs)?;
        order.cancel(reason)?;

        if let Some(tx_manager) = &self.tx_manager {
            return self.update_in_transaction(tx_manager, &order).await;
        }

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

    pub async fn auto_close_unpaid_orders(&self, timeout_secs: u64) -> AppResult<usize> {
        let cutoff = Utc::now() - Duration::seconds(timeout_secs as i64);
        let stores = self.store_repo.list().await?;
        let mut affected = 0usize;

        for store in stores {
            let orders = self.repo.list_by_store(store.id).await?;
            for mut order in orders {
                if order.status != axum_domain::runner_order::entity::RunnerOrderStatus::PendingPay
                {
                    continue;
                }
                if order.created_at > cutoff {
                    continue;
                }

                order.close_unpaid_timeout()?;
                if let Some(tx_manager) = &self.tx_manager {
                    self.update_in_transaction(tx_manager, &order).await?;
                } else {
                    self.repo.update(&order).await?;
                }
                affected += 1;
            }
        }
        Ok(affected)
    }

    pub async fn auto_accept_pending_orders(&self, timeout_secs: u64) -> AppResult<usize> {
        let cutoff = Utc::now() - Duration::seconds(timeout_secs as i64);
        let stores = self.store_repo.list().await?;
        let mut affected = 0usize;

        for store in stores {
            let orders = self.repo.list_by_store(store.id).await?;
            for mut order in orders {
                if order.status
                    != axum_domain::runner_order::entity::RunnerOrderStatus::PendingAccept
                {
                    continue;
                }
                let paid_at = order.pay_time.unwrap_or(order.created_at);
                if paid_at > cutoff {
                    continue;
                }

                order.admin_accept()?;
                if let Some(tx_manager) = &self.tx_manager {
                    self.update_in_transaction(tx_manager, &order).await?;
                } else {
                    self.repo.update(&order).await?;
                }
                affected += 1;
            }
        }
        Ok(affected)
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

fn ensure_cancel_within_window(
    created_at: chrono::DateTime<Utc>,
    pay_time: Option<chrono::DateTime<Utc>>,
    cancel_timeout_secs: u64,
) -> AppResult<()> {
    if cancel_timeout_secs == 0 {
        return Err(AppError::Validation("已超过可取消时间".into()));
    }

    let base = pay_time.unwrap_or(created_at);
    let elapsed = Utc::now().signed_duration_since(base).num_seconds();
    if elapsed >= cancel_timeout_secs as i64 {
        return Err(AppError::Validation("已超过可取消时间".into()));
    }
    Ok(())
}
