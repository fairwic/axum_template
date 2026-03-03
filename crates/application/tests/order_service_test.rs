use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum_application::{CreateGoodsOrderInput, OrderService};
use axum_common::AppResult;
use axum_domain::order::entity::{DeliveryType, GoodsOrder, GoodsOrderItem, GoodsOrderStatus};
use axum_domain::order::repo::GoodsOrderRepository;
use axum_domain::product::entity::{Product, ProductStatus};
use axum_domain::product::repo::ProductRepository;
use tokio::sync::Mutex;
use ulid::Ulid;

#[derive(Default)]
struct InMemoryOrderRepo {
    inner: Mutex<HashMap<Ulid, GoodsOrder>>,
}

#[async_trait]
impl GoodsOrderRepository for InMemoryOrderRepo {
    async fn create(&self, order: &GoodsOrder) -> AppResult<GoodsOrder> {
        let mut guard = self.inner.lock().await;
        guard.insert(order.id, order.clone());
        Ok(order.clone())
    }

    async fn update(&self, order: &GoodsOrder) -> AppResult<GoodsOrder> {
        let mut guard = self.inner.lock().await;
        guard.insert(order.id, order.clone());
        Ok(order.clone())
    }

    async fn find_by_id(&self, order_id: Ulid) -> AppResult<Option<GoodsOrder>> {
        let guard = self.inner.lock().await;
        Ok(guard.get(&order_id).cloned())
    }

    async fn list_by_user(&self, user_id: Ulid) -> AppResult<Vec<GoodsOrder>> {
        let guard = self.inner.lock().await;
        Ok(guard
            .values()
            .filter(|item| item.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn list_by_store(&self, store_id: Ulid) -> AppResult<Vec<GoodsOrder>> {
        let guard = self.inner.lock().await;
        Ok(guard
            .values()
            .filter(|item| item.store_id == store_id)
            .cloned()
            .collect())
    }
}

#[derive(Default)]
struct InMemoryProductRepo {
    inner: Mutex<HashMap<Ulid, Product>>,
}

#[async_trait]
impl ProductRepository for InMemoryProductRepo {
    async fn list_by_category(
        &self,
        _store_id: Ulid,
        _category_id: Ulid,
        _page: i64,
        _page_size: i64,
    ) -> AppResult<(Vec<Product>, i64)> {
        Ok((vec![], 0))
    }

    async fn search(
        &self,
        _store_id: Ulid,
        _keyword: &str,
        _page: i64,
        _page_size: i64,
    ) -> AppResult<(Vec<Product>, i64)> {
        Ok((vec![], 0))
    }

    async fn create(&self, product: &Product) -> AppResult<Product> {
        let mut guard = self.inner.lock().await;
        guard.insert(product.id, product.clone());
        Ok(product.clone())
    }

    async fn find_by_ids(&self, store_id: Ulid, product_ids: &[Ulid]) -> AppResult<Vec<Product>> {
        let guard = self.inner.lock().await;
        let mut result = Vec::new();
        for product_id in product_ids {
            if let Some(item) = guard.get(product_id) {
                if item.store_id == store_id {
                    result.push(item.clone());
                }
            }
        }
        Ok(result)
    }

    async fn try_lock_stock(&self, product_id: Ulid, qty: i32) -> AppResult<bool> {
        let mut guard = self.inner.lock().await;
        let Some(product) = guard.get_mut(&product_id) else {
            return Ok(false);
        };
        if product.status != ProductStatus::On || product.stock < qty {
            return Ok(false);
        }
        product.stock -= qty;
        Ok(true)
    }

    async fn release_stock(&self, product_id: Ulid, qty: i32) -> AppResult<()> {
        let mut guard = self.inner.lock().await;
        if let Some(product) = guard.get_mut(&product_id) {
            product.stock += qty;
        }
        Ok(())
    }
}

fn sample_item(product_id: Ulid) -> GoodsOrderItem {
    GoodsOrderItem {
        product_id,
        title_snapshot: "椰子水".into(),
        price_snapshot: 990,
        qty: 2,
    }
}

#[tokio::test]
async fn test_order_create_pay_cancel_flow() {
    let order_repo: Arc<dyn GoodsOrderRepository> = Arc::new(InMemoryOrderRepo::default());
    let product_repo: Arc<dyn ProductRepository> = Arc::new(InMemoryProductRepo::default());
    let service = OrderService::new(order_repo, product_repo.clone());

    let user_id = Ulid::new();
    let store_id = Ulid::new();
    let product = Product::new(
        store_id,
        Ulid::new(),
        "椰子水".into(),
        None,
        "img".into(),
        vec![],
        990,
        None,
        10,
        ProductStatus::On,
        vec![],
    )
    .unwrap();
    product_repo.create(&product).await.unwrap();

    let order = service
        .create(CreateGoodsOrderInput {
            user_id,
            store_id,
            delivery_type: DeliveryType::Delivery,
            items: vec![sample_item(product.id)],
            distance_km: Some(4.2),
            address_snapshot: Some(serde_json::json!({"detail":"A-101"})),
            store_snapshot: None,
            remark: Some("少冰".into()),
        })
        .await
        .unwrap();

    assert_eq!(order.status, GoodsOrderStatus::PendingPay);
    assert_eq!(order.amount_goods, 1980);
    assert_eq!(order.amount_delivery_fee, 200);

    let paid = service.pay(user_id, order.id).await.unwrap();
    assert_eq!(paid.status, GoodsOrderStatus::PendingAccept);

    let cancel_result = service.cancel(user_id, order.id, None).await;
    assert!(cancel_result.is_err());
}

#[tokio::test]
async fn test_order_admin_state_flow() {
    let order_repo: Arc<dyn GoodsOrderRepository> = Arc::new(InMemoryOrderRepo::default());
    let product_repo: Arc<dyn ProductRepository> = Arc::new(InMemoryProductRepo::default());
    let service = OrderService::new(order_repo, product_repo.clone());

    let user_id = Ulid::new();
    let store_id = Ulid::new();
    let product = Product::new(
        store_id,
        Ulid::new(),
        "椰子水".into(),
        None,
        "img".into(),
        vec![],
        990,
        None,
        10,
        ProductStatus::On,
        vec![],
    )
    .unwrap();
    product_repo.create(&product).await.unwrap();
    let order = service
        .create(CreateGoodsOrderInput {
            user_id,
            store_id,
            delivery_type: DeliveryType::Pickup,
            items: vec![sample_item(product.id)],
            distance_km: None,
            address_snapshot: None,
            store_snapshot: Some(serde_json::json!({"name":"店A"})),
            remark: None,
        })
        .await
        .unwrap();

    let paid = service.pay(user_id, order.id).await.unwrap();
    let accepted = service.admin_accept(paid.id).await.unwrap();
    assert_eq!(accepted.status, GoodsOrderStatus::Accepted);

    let dispatched = service.admin_dispatch(order.id).await.unwrap();
    assert_eq!(dispatched.status, GoodsOrderStatus::WaitingPickup);

    let completed = service.admin_complete(order.id).await.unwrap();
    assert_eq!(completed.status, GoodsOrderStatus::Completed);
}

#[tokio::test]
async fn test_order_create_fails_on_insufficient_stock() {
    let order_repo: Arc<dyn GoodsOrderRepository> = Arc::new(InMemoryOrderRepo::default());
    let product_repo: Arc<dyn ProductRepository> = Arc::new(InMemoryProductRepo::default());
    let service = OrderService::new(order_repo, product_repo.clone());

    let user_id = Ulid::new();
    let store_id = Ulid::new();
    let product = Product::new(
        store_id,
        Ulid::new(),
        "椰子水".into(),
        None,
        "img".into(),
        vec![],
        990,
        None,
        1,
        ProductStatus::On,
        vec![],
    )
    .unwrap();
    product_repo.create(&product).await.unwrap();

    let result = service
        .create(CreateGoodsOrderInput {
            user_id,
            store_id,
            delivery_type: DeliveryType::Delivery,
            items: vec![GoodsOrderItem {
                product_id: product.id,
                title_snapshot: "椰子水".into(),
                price_snapshot: 990,
                qty: 2,
            }],
            distance_km: Some(2.0),
            address_snapshot: Some(serde_json::json!({"detail":"A-101"})),
            store_snapshot: None,
            remark: None,
        })
        .await;

    assert!(result.is_err());
}
