//! Goods order service

use std::sync::Arc;
use std::collections::{HashMap, HashSet};

use axum_common::{AppError, AppResult};
use axum_domain::order::entity::{DeliveryType, GoodsOrder, GoodsOrderItem};
use axum_domain::order::repo::GoodsOrderRepository;
use axum_domain::product::entity::ProductStatus;
use axum_domain::product::repo::ProductRepository;
use ulid::Ulid;

const FREE_DELIVERY_RADIUS_KM: f64 = 3.0;
const DELIVERY_FEE_PER_EXTRA_KM: i32 = 100;

#[derive(Debug, Clone)]
pub struct CreateGoodsOrderInput {
    pub user_id: Ulid,
    pub store_id: Ulid,
    pub delivery_type: DeliveryType,
    pub items: Vec<GoodsOrderItem>,
    pub distance_km: Option<f64>,
    pub address_snapshot: Option<serde_json::Value>,
    pub store_snapshot: Option<serde_json::Value>,
    pub remark: Option<String>,
}

#[derive(Clone)]
pub struct OrderService {
    repo: Arc<dyn GoodsOrderRepository>,
    product_repo: Arc<dyn ProductRepository>,
}

impl OrderService {
    pub fn new(repo: Arc<dyn GoodsOrderRepository>, product_repo: Arc<dyn ProductRepository>) -> Self {
        Self { repo, product_repo }
    }

    pub async fn create(&self, input: CreateGoodsOrderInput) -> AppResult<GoodsOrder> {
        let checked_items = self.recheck_items(input.store_id, &input.items).await?;
        let delivery_fee = calc_delivery_fee(&input.delivery_type, input.distance_km);
        let mut locked_items: Vec<(Ulid, i32)> = Vec::with_capacity(checked_items.len());
        for item in &checked_items {
            let locked = self
                .product_repo
                .try_lock_stock(item.product_id, item.qty)
                .await?;
            if !locked {
                self.rollback_locked_items(&locked_items).await;
                return Err(AppError::Validation("库存变化，请调整后再试".into()));
            }
            locked_items.push((item.product_id, item.qty));
        }

        let order = GoodsOrder::new(
            input.user_id,
            input.store_id,
            input.delivery_type,
            checked_items,
            delivery_fee,
            input.distance_km,
            input.address_snapshot,
            input.store_snapshot,
            input.remark,
        )?;
        match self.repo.create(&order).await {
            Ok(saved) => Ok(saved),
            Err(err) => {
                self.rollback_locked_items(&locked_items).await;
                Err(err)
            }
        }
    }

    pub async fn pay(&self, user_id: Ulid, order_id: Ulid) -> AppResult<GoodsOrder> {
        let mut order = self.must_get_for_user(user_id, order_id).await?;
        order.mark_paid()?;
        self.repo.update(&order).await
    }

    pub async fn cancel(
        &self,
        user_id: Ulid,
        order_id: Ulid,
        reason: Option<String>,
    ) -> AppResult<GoodsOrder> {
        let mut order = self.must_get_for_user(user_id, order_id).await?;
        order.cancel(reason)?;
        let updated = self.repo.update(&order).await?;
        for item in &updated.items {
            self.product_repo.release_stock(item.product_id, item.qty).await?;
        }
        Ok(updated)
    }

    pub async fn list_by_user(&self, user_id: Ulid) -> AppResult<Vec<GoodsOrder>> {
        self.repo.list_by_user(user_id).await
    }

    pub async fn get_by_user(&self, user_id: Ulid, order_id: Ulid) -> AppResult<GoodsOrder> {
        self.must_get_for_user(user_id, order_id).await
    }

    pub async fn admin_list_by_store(&self, store_id: Ulid) -> AppResult<Vec<GoodsOrder>> {
        self.repo.list_by_store(store_id).await
    }

    pub async fn admin_accept(&self, order_id: Ulid) -> AppResult<GoodsOrder> {
        let mut order = self.must_get(order_id).await?;
        order.admin_accept()?;
        self.repo.update(&order).await
    }

    pub async fn admin_dispatch(&self, order_id: Ulid) -> AppResult<GoodsOrder> {
        let mut order = self.must_get(order_id).await?;
        order.admin_dispatch()?;
        self.repo.update(&order).await
    }

    pub async fn admin_complete(&self, order_id: Ulid) -> AppResult<GoodsOrder> {
        let mut order = self.must_get(order_id).await?;
        order.admin_complete()?;
        self.repo.update(&order).await
    }

    async fn must_get(&self, order_id: Ulid) -> AppResult<GoodsOrder> {
        self.repo
            .find_by_id(order_id)
            .await?
            .ok_or_else(|| AppError::NotFound("order not found".into()))
    }

    async fn must_get_for_user(&self, user_id: Ulid, order_id: Ulid) -> AppResult<GoodsOrder> {
        let order = self.must_get(order_id).await?;
        if order.user_id != user_id {
            return Err(AppError::Forbidden);
        }
        Ok(order)
    }

    async fn recheck_items(
        &self,
        store_id: Ulid,
        items: &[GoodsOrderItem],
    ) -> AppResult<Vec<GoodsOrderItem>> {
        if items.is_empty() {
            return Err(AppError::Validation("order items required".into()));
        }

        let mut seen = HashSet::new();
        let mut ids = Vec::with_capacity(items.len());
        for item in items {
            if !seen.insert(item.product_id) {
                return Err(AppError::Validation("duplicate product in order".into()));
            }
            ids.push(item.product_id);
        }

        let products = self.product_repo.find_by_ids(store_id, &ids).await?;
        if products.len() != ids.len() {
            return Err(AppError::Validation("商品不存在或已下架".into()));
        }

        let product_map: HashMap<Ulid, axum_domain::Product> = products
            .into_iter()
            .map(|item| (item.id, item))
            .collect();

        let mut checked = Vec::with_capacity(items.len());
        for item in items {
            let product = product_map
                .get(&item.product_id)
                .ok_or_else(|| AppError::Validation("商品不存在或已下架".into()))?;
            if product.status != ProductStatus::On {
                return Err(AppError::Validation("商品已下架".into()));
            }
            if item.qty <= 0 {
                return Err(AppError::Validation("商品数量必须大于0".into()));
            }
            if product.stock < item.qty {
                return Err(AppError::Validation("库存不足，请调整后再试".into()));
            }
            if item.price_snapshot != product.price {
                return Err(AppError::Validation("商品价格已变化，请刷新后重试".into()));
            }

            checked.push(GoodsOrderItem {
                product_id: product.id,
                title_snapshot: product.title.clone(),
                price_snapshot: product.price,
                qty: item.qty,
            });
        }

        Ok(checked)
    }

    async fn rollback_locked_items(&self, items: &[(Ulid, i32)]) {
        for (product_id, qty) in items {
            let _ = self.product_repo.release_stock(*product_id, *qty).await;
        }
    }
}

fn calc_delivery_fee(delivery_type: &DeliveryType, distance_km: Option<f64>) -> i32 {
    if *delivery_type != DeliveryType::Delivery {
        return 0;
    }

    let distance_km = distance_km.unwrap_or(0.0);
    if distance_km <= FREE_DELIVERY_RADIUS_KM {
        return 0;
    }

    let extra_km = (distance_km - FREE_DELIVERY_RADIUS_KM).ceil() as i32;
    extra_km * DELIVERY_FEE_PER_EXTRA_KM
}
