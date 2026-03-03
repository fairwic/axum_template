use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum_application::{CreateGoodsOrderInput, OrderService};
use axum_common::AppResult;
use axum_domain::order::entity::{
    DeliveryType, GoodsOrder, GoodsOrderItem, GoodsOrderStatus, PayStatus,
};
use axum_domain::order::repo::GoodsOrderRepository;
use axum_domain::product::entity::{Product, ProductStatus};
use axum_domain::product::repo::ProductRepository;
use axum_domain::store::entity::{Store, StoreStatus};
use axum_domain::store::repo::StoreRepository;
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

    async fn find_by_id(&self, store_id: Ulid) -> AppResult<Option<Store>> {
        let guard = self.inner.lock().await;
        Ok(guard.get(&store_id).cloned())
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

fn sample_store() -> Store {
    Store::new(
        "店A".into(),
        "A区".into(),
        30.0,
        120.0,
        "13800000000".into(),
        "9:00-22:00".into(),
        StoreStatus::Open,
        3.0,
        100,
        100,
        200,
    )
    .unwrap()
}

#[tokio::test]
async fn test_order_create_pay_cancel_flow() {
    let order_repo: Arc<dyn GoodsOrderRepository> = Arc::new(InMemoryOrderRepo::default());
    let product_repo: Arc<dyn ProductRepository> = Arc::new(InMemoryProductRepo::default());
    let store_repo: Arc<dyn StoreRepository> = Arc::new(InMemoryStoreRepo::default());
    let service = OrderService::new(order_repo, product_repo.clone(), store_repo.clone());

    let user_id = Ulid::new();
    let store = sample_store();
    let store_id = store.id;
    store_repo.create(&store).await.unwrap();
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
    assert_eq!(order.amount_delivery_fee, 300);

    let paid = service.pay(user_id, order.id).await.unwrap();
    assert_eq!(paid.status, GoodsOrderStatus::PendingAccept);

    let canceled = service.cancel(user_id, order.id, None).await.unwrap();
    assert_eq!(canceled.status, GoodsOrderStatus::Canceled);
    assert_eq!(canceled.pay_status, PayStatus::Refunded);
}

#[tokio::test]
async fn test_order_admin_state_flow() {
    let order_repo: Arc<dyn GoodsOrderRepository> = Arc::new(InMemoryOrderRepo::default());
    let product_repo: Arc<dyn ProductRepository> = Arc::new(InMemoryProductRepo::default());
    let store_repo: Arc<dyn StoreRepository> = Arc::new(InMemoryStoreRepo::default());
    let service = OrderService::new(order_repo, product_repo.clone(), store_repo.clone());

    let user_id = Ulid::new();
    let store = sample_store();
    let store_id = store.id;
    store_repo.create(&store).await.unwrap();
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
    let store_repo: Arc<dyn StoreRepository> = Arc::new(InMemoryStoreRepo::default());
    let service = OrderService::new(order_repo, product_repo.clone(), store_repo.clone());

    let user_id = Ulid::new();
    let store = sample_store();
    let store_id = store.id;
    store_repo.create(&store).await.unwrap();
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

#[tokio::test]
async fn test_order_preview_returns_config_based_delivery_fee() {
    let order_repo: Arc<dyn GoodsOrderRepository> = Arc::new(InMemoryOrderRepo::default());
    let product_repo: Arc<dyn ProductRepository> = Arc::new(InMemoryProductRepo::default());
    let store_repo: Arc<dyn StoreRepository> = Arc::new(InMemoryStoreRepo::default());
    let service = OrderService::new(order_repo, product_repo.clone(), store_repo.clone());

    let store = sample_store();
    let store_id = store.id;
    store_repo.create(&store).await.unwrap();

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

    let preview = service
        .preview(
            store_id,
            DeliveryType::Delivery,
            vec![sample_item(product.id)],
            Some(4.2),
        )
        .await
        .unwrap();

    assert_eq!(preview.amount_goods, 1980);
    assert_eq!(preview.amount_delivery_fee, 300);
    assert_eq!(preview.amount_payable, 2280);
    assert!(preview.deliverable);
}

#[tokio::test]
async fn test_order_auto_close_unpaid_releases_stock() {
    let order_repo: Arc<dyn GoodsOrderRepository> = Arc::new(InMemoryOrderRepo::default());
    let product_repo: Arc<dyn ProductRepository> = Arc::new(InMemoryProductRepo::default());
    let store_repo: Arc<dyn StoreRepository> = Arc::new(InMemoryStoreRepo::default());
    let service = OrderService::new(order_repo, product_repo.clone(), store_repo.clone());

    let user_id = Ulid::new();
    let store = sample_store();
    let store_id = store.id;
    store_repo.create(&store).await.unwrap();
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

    let created = service
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
        .await
        .unwrap();
    assert_eq!(created.status, GoodsOrderStatus::PendingPay);

    let closed_count = service.auto_close_unpaid_orders(0).await.unwrap();
    assert_eq!(closed_count, 1);

    let closed = service.get_by_user(user_id, created.id).await.unwrap();
    assert_eq!(closed.status, GoodsOrderStatus::Closed);
    assert_eq!(closed.pay_status, PayStatus::Unpaid);

    let product_after = product_repo
        .find_by_ids(store_id, &[product.id])
        .await
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(product_after.stock, 10);
}

#[tokio::test]
async fn test_order_auto_accept_pending_orders() {
    let order_repo: Arc<dyn GoodsOrderRepository> = Arc::new(InMemoryOrderRepo::default());
    let product_repo: Arc<dyn ProductRepository> = Arc::new(InMemoryProductRepo::default());
    let store_repo: Arc<dyn StoreRepository> = Arc::new(InMemoryStoreRepo::default());
    let service = OrderService::new(order_repo, product_repo.clone(), store_repo.clone());

    let user_id = Ulid::new();
    let store = sample_store();
    let store_id = store.id;
    store_repo.create(&store).await.unwrap();
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

    let created = service
        .create(CreateGoodsOrderInput {
            user_id,
            store_id,
            delivery_type: DeliveryType::Delivery,
            items: vec![GoodsOrderItem {
                product_id: product.id,
                title_snapshot: "椰子水".into(),
                price_snapshot: 990,
                qty: 1,
            }],
            distance_km: Some(2.0),
            address_snapshot: Some(serde_json::json!({"detail":"A-101"})),
            store_snapshot: None,
            remark: None,
        })
        .await
        .unwrap();
    let paid = service.pay(user_id, created.id).await.unwrap();
    assert_eq!(paid.status, GoodsOrderStatus::PendingAccept);

    let accepted_count = service.auto_accept_pending_orders(0).await.unwrap();
    assert_eq!(accepted_count, 1);

    let accepted = service.get_by_user(user_id, created.id).await.unwrap();
    assert_eq!(accepted.status, GoodsOrderStatus::Accepted);
}

#[tokio::test]
async fn test_order_repurchase_creates_new_pending_pay_order() {
    let order_repo: Arc<dyn GoodsOrderRepository> = Arc::new(InMemoryOrderRepo::default());
    let product_repo: Arc<dyn ProductRepository> = Arc::new(InMemoryProductRepo::default());
    let store_repo: Arc<dyn StoreRepository> = Arc::new(InMemoryStoreRepo::default());
    let service = OrderService::new(order_repo, product_repo.clone(), store_repo.clone());

    let user_id = Ulid::new();
    let store = sample_store();
    let store_id = store.id;
    store_repo.create(&store).await.unwrap();
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

    let origin = service
        .create(CreateGoodsOrderInput {
            user_id,
            store_id,
            delivery_type: DeliveryType::Delivery,
            items: vec![GoodsOrderItem {
                product_id: product.id,
                title_snapshot: "椰子水".into(),
                price_snapshot: 990,
                qty: 1,
            }],
            distance_km: Some(2.0),
            address_snapshot: Some(serde_json::json!({"detail":"A-101"})),
            store_snapshot: None,
            remark: Some("不要冰".into()),
        })
        .await
        .unwrap();

    let repurchased = service.repurchase(user_id, origin.id).await.unwrap();
    assert_ne!(repurchased.id, origin.id);
    assert_eq!(repurchased.status, GoodsOrderStatus::PendingPay);
    assert_eq!(repurchased.pay_status, PayStatus::Unpaid);
    assert_eq!(repurchased.items.len(), 1);
    assert_eq!(repurchased.items[0].qty, 1);
    assert_eq!(repurchased.amount_payable, 990);
}
