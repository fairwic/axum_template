//! Goods order service

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::dtos::order_dto::{CreateGoodsOrderInput, OrderPreview};
use axum_common::{AppError, AppResult};
use axum_domain::order::entity::{DeliveryType, GoodsOrder, GoodsOrderItem};
use axum_domain::order::repo::GoodsOrderRepository;
use axum_domain::product::entity::ProductStatus;
use axum_domain::product::repo::ProductRepository;
use axum_domain::store::repo::StoreRepository;
use ulid::Ulid;

#[derive(Clone)]
pub struct OrderService {
    repo: Arc<dyn GoodsOrderRepository>,
    product_repo: Arc<dyn ProductRepository>,
    store_repo: Arc<dyn StoreRepository>,
}

impl OrderService {
    pub fn new(
        repo: Arc<dyn GoodsOrderRepository>,
        product_repo: Arc<dyn ProductRepository>,
        store_repo: Arc<dyn StoreRepository>,
    ) -> Self {
        Self {
            repo,
            product_repo,
            store_repo,
        }
    }

    pub async fn create(&self, input: CreateGoodsOrderInput) -> AppResult<GoodsOrder> {
        let checked_items = self.recheck_items(input.store_id, &input.items).await?;
        let preview = self
            .preview_from_checked(
                input.store_id,
                input.delivery_type.clone(),
                &checked_items,
                input.distance_km,
            )
            .await?;
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
            preview.amount_delivery_fee,
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

    pub async fn preview(
        &self,
        store_id: Ulid,
        delivery_type: DeliveryType,
        items: Vec<GoodsOrderItem>,
        distance_km: Option<f64>,
    ) -> AppResult<OrderPreview> {
        let checked_items = self.recheck_items(store_id, &items).await?;
        self.preview_from_checked(store_id, delivery_type, &checked_items, distance_km)
            .await
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
            self.product_repo
                .release_stock(item.product_id, item.qty)
                .await?;
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

        let product_map: HashMap<Ulid, axum_domain::Product> =
            products.into_iter().map(|item| (item.id, item)).collect();

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

    async fn preview_from_checked(
        &self,
        store_id: Ulid,
        delivery_type: DeliveryType,
        items: &[GoodsOrderItem],
        distance_km: Option<f64>,
    ) -> AppResult<OrderPreview> {
        let amount_goods = items
            .iter()
            .map(|item| item.price_snapshot * item.qty)
            .sum::<i32>();
        let amount_delivery_fee = self
            .calc_delivery_fee(store_id, &delivery_type, distance_km)
            .await?;
        let amount_discount = 0;
        let amount_payable = amount_goods + amount_delivery_fee - amount_discount;

        Ok(OrderPreview {
            amount_goods,
            amount_delivery_fee,
            amount_discount,
            amount_payable,
            distance_km,
            deliverable: true,
        })
    }

    async fn calc_delivery_fee(
        &self,
        store_id: Ulid,
        delivery_type: &DeliveryType,
        distance_km: Option<f64>,
    ) -> AppResult<i32> {
        if *delivery_type != DeliveryType::Delivery {
            return Ok(0);
        }

        let distance_km = distance_km
            .ok_or_else(|| AppError::Validation("distance_km required for delivery".into()))?;
        let store = self
            .store_repo
            .find_by_id(store_id)
            .await?
            .ok_or_else(|| AppError::NotFound("store not found".into()))?;

        if distance_km <= store.delivery_radius_km {
            return Ok(0);
        }

        let extra_km = (distance_km - store.delivery_radius_km).ceil() as i32;
        Ok(store.delivery_fee_base + extra_km * store.delivery_fee_per_km)
    }
}
