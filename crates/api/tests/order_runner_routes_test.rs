use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    body::{to_bytes, Body},
    http::Request,
};
use axum_api::auth::jwt::{encode_token, Claims};
use axum_api::{create_router, AppState};
use axum_application::{
    AddressService, AdminService, CartService, CategoryService, OrderService, ProductService,
    RunnerOrderService, StoreService, UserService,
};
use axum_core_kernel::AppResult;
use axum_domain::address::entity::Address;
use axum_domain::address::repo::AddressRepository;
use axum_domain::admin::entity::Admin;
use axum_domain::admin::repo::AdminRepository;
use axum_domain::cart::entity::Cart;
use axum_domain::cart::repo::CartRepository;
use axum_domain::category::entity::Category;
use axum_domain::category::repo::CategoryRepository;
use axum_domain::order::repo::GoodsOrderRepository;
use axum_domain::product::entity::{Product, ProductStatus};
use axum_domain::product::repo::ProductRepository;
use axum_domain::runner_order::repo::RunnerOrderRepository;
use axum_domain::store::entity::Store;
use axum_domain::store::repo::StoreRepository;
use axum_domain::user::repo::UserRepository;
use axum_domain::{GoodsOrder, RunnerOrder, User};
use chrono::Utc;
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tower::util::ServiceExt;
use ulid::Ulid;

#[derive(Default)]
struct InMemoryUserRepo {
    inner: Mutex<HashMap<String, User>>,
}

#[async_trait]
impl UserRepository for InMemoryUserRepo {
    async fn find_by_openid(&self, openid: &str) -> AppResult<Option<User>> {
        let guard = self.inner.lock().await;
        Ok(guard.get(openid).cloned())
    }

    async fn create(&self, user: &User) -> AppResult<User> {
        let mut guard = self.inner.lock().await;
        guard.insert(user.openid.clone(), user.clone());
        Ok(user.clone())
    }

    async fn find_by_id(&self, user_id: Ulid) -> AppResult<Option<User>> {
        let guard = self.inner.lock().await;
        Ok(guard.values().find(|item| item.id == user_id).cloned())
    }

    async fn find_by_phone(&self, phone: &str) -> AppResult<Option<User>> {
        let guard = self.inner.lock().await;
        Ok(guard
            .values()
            .find(|item| item.phone.as_deref() == Some(phone))
            .cloned())
    }

    async fn set_current_store(&self, user_id: Ulid, store_id: Ulid) -> AppResult<User> {
        let mut guard = self.inner.lock().await;
        let key = guard
            .iter()
            .find_map(|(openid, user)| (user.id == user_id).then(|| openid.clone()));
        let Some(key) = key else {
            return Err(axum_core_kernel::AppError::NotFound(
                "user not found".into(),
            ));
        };
        let user = guard
            .get_mut(&key)
            .ok_or_else(|| axum_core_kernel::AppError::NotFound("user not found".into()))?;
        user.current_store_id = Some(store_id);
        Ok(user.clone())
    }

    async fn bind_phone(&self, user_id: Ulid, phone: String) -> AppResult<User> {
        let mut guard = self.inner.lock().await;
        let key = guard
            .iter()
            .find_map(|(openid, user)| (user.id == user_id).then(|| openid.clone()));
        let Some(key) = key else {
            return Err(axum_core_kernel::AppError::NotFound(
                "user not found".into(),
            ));
        };
        let user = guard
            .get_mut(&key)
            .ok_or_else(|| axum_core_kernel::AppError::NotFound("user not found".into()))?;
        user.phone = Some(phone);
        Ok(user.clone())
    }
}

#[derive(Default)]
struct InMemoryAdminRepo {
    inner: Mutex<HashMap<String, Admin>>,
}

#[async_trait]
impl AdminRepository for InMemoryAdminRepo {
    async fn find_by_phone(&self, phone: &str) -> AppResult<Option<Admin>> {
        let guard = self.inner.lock().await;
        Ok(guard.get(phone).cloned())
    }

    async fn create(&self, admin: &Admin) -> AppResult<Admin> {
        let mut guard = self.inner.lock().await;
        guard.insert(admin.phone.clone(), admin.clone());
        Ok(admin.clone())
    }
}

#[derive(Default)]
struct InMemoryStoreRepo {
    inner: Mutex<HashMap<String, Store>>,
}

#[async_trait]
impl StoreRepository for InMemoryStoreRepo {
    async fn list(&self) -> AppResult<Vec<Store>> {
        let guard = self.inner.lock().await;
        Ok(guard.values().cloned().collect())
    }

    async fn create(&self, store: &Store) -> AppResult<Store> {
        let mut guard = self.inner.lock().await;
        guard.insert(store.id.to_string(), store.clone());
        Ok(store.clone())
    }

    async fn find_by_id(&self, store_id: Ulid) -> AppResult<Option<Store>> {
        let guard = self.inner.lock().await;
        Ok(guard.get(&store_id.to_string()).cloned())
    }

    async fn update(&self, store: &Store) -> AppResult<Store> {
        let mut guard = self.inner.lock().await;
        guard.insert(store.id.to_string(), store.clone());
        Ok(store.clone())
    }
}

#[derive(Default)]
struct InMemoryCategoryRepo {
    inner: Mutex<HashMap<String, Category>>,
}

#[async_trait]
impl CategoryRepository for InMemoryCategoryRepo {
    async fn list_by_store(&self, store_id: Ulid) -> AppResult<Vec<Category>> {
        let guard = self.inner.lock().await;
        Ok(guard
            .values()
            .filter(|item| item.store_id == store_id)
            .cloned()
            .collect())
    }

    async fn create(&self, category: &Category) -> AppResult<Category> {
        let mut guard = self.inner.lock().await;
        guard.insert(category.id.to_string(), category.clone());
        Ok(category.clone())
    }

    async fn find_by_id(&self, category_id: Ulid) -> AppResult<Option<Category>> {
        let guard = self.inner.lock().await;
        Ok(guard.get(&category_id.to_string()).cloned())
    }

    async fn update(&self, category: &Category) -> AppResult<Category> {
        let mut guard = self.inner.lock().await;
        guard.insert(category.id.to_string(), category.clone());
        Ok(category.clone())
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

    async fn update(&self, product: &Product) -> AppResult<Product> {
        let mut guard = self.inner.lock().await;
        guard.insert(product.id, product.clone());
        Ok(product.clone())
    }

    async fn find_by_id(&self, store_id: Ulid, product_id: Ulid) -> AppResult<Option<Product>> {
        let guard = self.inner.lock().await;
        Ok(guard
            .get(&product_id)
            .filter(|item| item.store_id == store_id)
            .cloned())
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
struct InMemoryCartRepo {
    inner: Mutex<HashMap<(Ulid, Ulid), Cart>>,
}

#[async_trait]
impl CartRepository for InMemoryCartRepo {
    async fn get_cart(&self, user_id: Ulid, store_id: Ulid) -> AppResult<Option<Cart>> {
        let guard = self.inner.lock().await;
        Ok(guard.get(&(user_id, store_id)).cloned())
    }

    async fn create_cart(&self, user_id: Ulid, store_id: Ulid) -> AppResult<Cart> {
        let mut guard = self.inner.lock().await;
        let cart = Cart::new(user_id, store_id);
        guard.insert((user_id, store_id), cart.clone());
        Ok(cart)
    }

    async fn upsert_item(
        &self,
        user_id: Ulid,
        store_id: Ulid,
        product_id: Ulid,
        qty: i32,
        price_snapshot: i32,
    ) -> AppResult<()> {
        let mut guard = self.inner.lock().await;
        let cart = guard
            .entry((user_id, store_id))
            .or_insert_with(|| Cart::new(user_id, store_id));
        cart.upsert_item(product_id, qty, price_snapshot);
        Ok(())
    }

    async fn remove_item(&self, user_id: Ulid, store_id: Ulid, product_id: Ulid) -> AppResult<()> {
        let mut guard = self.inner.lock().await;
        if let Some(cart) = guard.get_mut(&(user_id, store_id)) {
            cart.remove_item(product_id);
        }
        Ok(())
    }

    async fn clear(&self, user_id: Ulid, store_id: Ulid) -> AppResult<()> {
        let mut guard = self.inner.lock().await;
        if let Some(cart) = guard.get_mut(&(user_id, store_id)) {
            cart.items.clear();
        }
        Ok(())
    }
}

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

#[derive(Default)]
struct InMemoryAddressRepo {
    inner: Mutex<HashMap<Ulid, Address>>,
}

#[async_trait]
impl AddressRepository for InMemoryAddressRepo {
    async fn list_by_user(&self, user_id: Ulid) -> AppResult<Vec<Address>> {
        let guard = self.inner.lock().await;
        Ok(guard
            .values()
            .filter(|item| item.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn find_by_id(&self, user_id: Ulid, address_id: Ulid) -> AppResult<Option<Address>> {
        let guard = self.inner.lock().await;
        Ok(guard
            .get(&address_id)
            .cloned()
            .filter(|item| item.user_id == user_id))
    }

    async fn create(&self, address: &Address) -> AppResult<Address> {
        let mut guard = self.inner.lock().await;
        guard.insert(address.id, address.clone());
        Ok(address.clone())
    }

    async fn update(&self, address: &Address) -> AppResult<Address> {
        let mut guard = self.inner.lock().await;
        guard.insert(address.id, address.clone());
        Ok(address.clone())
    }

    async fn delete(&self, user_id: Ulid, address_id: Ulid) -> AppResult<()> {
        let mut guard = self.inner.lock().await;
        if guard
            .get(&address_id)
            .map(|item| item.user_id == user_id)
            .unwrap_or(false)
        {
            guard.remove(&address_id);
        }
        Ok(())
    }
}

#[derive(Default)]
struct FakeLbs;

#[async_trait]
impl axum_application::services::store_service::LbsService for FakeLbs {
    async fn distance_km(&self, _from: (f64, f64), _to: (f64, f64)) -> AppResult<f64> {
        Ok(0.0)
    }
}

fn user_auth_header(user_id: Ulid) -> String {
    let claims = Claims {
        sub: user_id.to_string(),
        role: "USER".into(),
        exp: (Utc::now().timestamp() + 3600) as usize,
    };
    let token = encode_token(&claims, "secret").unwrap();
    format!("Bearer {token}")
}

fn admin_auth_header() -> String {
    let claims = Claims {
        sub: Ulid::new().to_string(),
        role: "PLATFORM".into(),
        exp: (Utc::now().timestamp() + 3600) as usize,
    };
    let token = encode_token(&claims, "secret").unwrap();
    format!("Bearer {token}")
}

async fn create_test_app(store_id: Ulid, product_id: Ulid) -> axum::Router {
    let user_repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepo::default());
    let admin_repo: Arc<dyn AdminRepository> = Arc::new(InMemoryAdminRepo::default());
    let store_repo: Arc<dyn StoreRepository> = Arc::new(InMemoryStoreRepo::default());
    let category_repo: Arc<dyn CategoryRepository> = Arc::new(InMemoryCategoryRepo::default());
    let product_repo_impl = Arc::new(InMemoryProductRepo::default());
    let product_repo: Arc<dyn ProductRepository> = product_repo_impl.clone();
    let cart_repo: Arc<dyn CartRepository> = Arc::new(InMemoryCartRepo::default());
    let order_repo: Arc<dyn GoodsOrderRepository> = Arc::new(InMemoryOrderRepo::default());
    let runner_repo: Arc<dyn RunnerOrderRepository> = Arc::new(InMemoryRunnerOrderRepo::default());
    let address_repo: Arc<dyn AddressRepository> = Arc::new(InMemoryAddressRepo::default());

    let mut seed_product = Product::new(
        store_id,
        Ulid::new(),
        "seed".into(),
        None,
        "img".into(),
        vec![],
        990,
        None,
        200,
        ProductStatus::On,
        vec![],
    )
    .unwrap();
    seed_product.id = product_id;
    let mut guard = product_repo_impl.inner.lock().await;
    guard.insert(seed_product.id, seed_product.clone());
    drop(guard);

    let seed_store = Store::new(
        "店A".into(),
        "A区".into(),
        30.0,
        120.0,
        "13800000000".into(),
        "9:00-22:00".into(),
        axum_domain::store::entity::StoreStatus::Open,
        3.0,
        100,
        100,
        200,
    )
    .unwrap();
    store_repo
        .create(&Store {
            id: store_id,
            ..seed_store
        })
        .await
        .unwrap();

    let state = AppState::new(
        UserService::new(user_repo),
        AdminService::new(admin_repo),
        StoreService::new(store_repo.clone(), Arc::new(FakeLbs)),
        CategoryService::new(category_repo),
        ProductService::new(product_repo.clone()),
        CartService::new(cart_repo),
        "secret".into(),
        3600,
        300,
    )
    .with_address_service(AddressService::new(address_repo))
    .with_order_services(
        OrderService::new(order_repo, product_repo, store_repo.clone()),
        RunnerOrderService::new(runner_repo, store_repo),
    );

    create_router(state)
}

#[tokio::test]
async fn test_goods_order_and_runner_order_flow() {
    let user_id = Ulid::new();
    let store_id = Ulid::new();
    let product_id = Ulid::new();
    let app = create_test_app(store_id, product_id).await;
    let user_auth = user_auth_header(user_id);
    let admin_auth = admin_auth_header();

    let create_address_req = Request::builder()
        .method("POST")
        .uri("/api/v1/addresses")
        .header("content-type", "application/json")
        .header("authorization", user_auth.clone())
        .body(Body::from(
            json!({
                "name":"张三",
                "phone":"13800000000",
                "detail":"A101",
                "lat":30.0,
                "lng":120.0,
                "is_default":true
            })
            .to_string(),
        ))
        .unwrap();
    let create_address_res = app.clone().oneshot(create_address_req).await.unwrap();
    let create_address_body = to_bytes(create_address_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let create_address_value: Value = serde_json::from_slice(&create_address_body).unwrap();
    assert_eq!(create_address_value["success"], true);
    let address_id = create_address_value["data"]["address_id"]
        .as_str()
        .unwrap()
        .to_string();

    let preview_order_req = Request::builder()
        .method("POST")
        .uri("/api/v1/orders/preview")
        .header("content-type", "application/json")
        .header("authorization", user_auth.clone())
        .body(Body::from(
            json!({
                "store_id": store_id.to_string(),
                "delivery_type": "DELIVERY",
                "items": [{
                    "product_id": product_id.to_string(),
                    "title_snapshot": "椰子水",
                    "price_snapshot": 990,
                    "qty": 1
                }],
                "distance_km": 4.2
            })
            .to_string(),
        ))
        .unwrap();
    let preview_order_res = app.clone().oneshot(preview_order_req).await.unwrap();
    let preview_order_body = to_bytes(preview_order_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let preview_order_value: Value = serde_json::from_slice(&preview_order_body).unwrap();
    assert_eq!(preview_order_value["success"], true);
    assert_eq!(preview_order_value["data"]["amount_delivery_fee"], 300);
    assert_eq!(preview_order_value["data"]["amount_payable"], 1290);

    let create_order_payload = json!({
        "store_id": store_id.to_string(),
        "delivery_type": "DELIVERY",
        "items": [{
            "product_id": product_id.to_string(),
            "title_snapshot": "椰子水",
            "price_snapshot": 990,
            "qty": 1
        }],
        "distance_km": 4.2,
        "address_id": address_id
    });

    let create_order_req = Request::builder()
        .method("POST")
        .uri("/api/v1/orders/create")
        .header("content-type", "application/json")
        .header("authorization", user_auth.clone())
        .body(Body::from(create_order_payload.to_string()))
        .unwrap();
    let create_order_res = app.clone().oneshot(create_order_req).await.unwrap();
    let create_order_body = to_bytes(create_order_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let create_order_value: Value = serde_json::from_slice(&create_order_body).unwrap();
    assert_eq!(create_order_value["success"], true);

    let order_id = create_order_value["data"]["order_id"]
        .as_str()
        .unwrap()
        .to_string();
    let pay_order_req = Request::builder()
        .method("POST")
        .uri("/api/v1/orders/pay")
        .header("content-type", "application/json")
        .header("authorization", user_auth.clone())
        .body(Body::from(json!({"order_id": order_id}).to_string()))
        .unwrap();
    let pay_order_res = app.clone().oneshot(pay_order_req).await.unwrap();
    let pay_order_body = to_bytes(pay_order_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let pay_order_value: Value = serde_json::from_slice(&pay_order_body).unwrap();
    assert_eq!(pay_order_value["success"], true);
    assert_eq!(pay_order_value["data"]["status"], "PENDING_ACCEPT");

    let cancel_paid_req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/orders/{}/cancel", order_id))
        .header("content-type", "application/json")
        .header("authorization", user_auth.clone())
        .body(Body::from(r#"{"reason":"不想要了"}"#))
        .unwrap();
    let cancel_paid_res = app.clone().oneshot(cancel_paid_req).await.unwrap();
    let cancel_paid_body = to_bytes(cancel_paid_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let cancel_paid_value: Value = serde_json::from_slice(&cancel_paid_body).unwrap();
    assert_eq!(cancel_paid_value["success"], true);
    assert_eq!(cancel_paid_value["data"]["status"], "CANCELED");
    assert_eq!(cancel_paid_value["data"]["pay_status"], "REFUNDED");

    let create_order_unpaid_payload = json!({
        "store_id": store_id.to_string(),
        "delivery_type": "DELIVERY",
        "items": [{
            "product_id": product_id.to_string(),
            "title_snapshot": "椰子水",
            "price_snapshot": 990,
            "qty": 1
        }],
        "distance_km": 2.0,
        "address_id": address_id
    });
    let create_order_unpaid_req = Request::builder()
        .method("POST")
        .uri("/api/v1/orders/create")
        .header("content-type", "application/json")
        .header("authorization", user_auth.clone())
        .body(Body::from(create_order_unpaid_payload.to_string()))
        .unwrap();
    let create_order_unpaid_res = app.clone().oneshot(create_order_unpaid_req).await.unwrap();
    let create_order_unpaid_body = to_bytes(create_order_unpaid_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let create_order_unpaid_value: Value =
        serde_json::from_slice(&create_order_unpaid_body).unwrap();
    assert_eq!(create_order_unpaid_value["success"], true);
    let unpaid_order_id = create_order_unpaid_value["data"]["order_id"]
        .as_str()
        .unwrap()
        .to_string();

    let cancel_unpaid_req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/orders/{}/cancel", unpaid_order_id))
        .header("content-type", "application/json")
        .header("authorization", user_auth.clone())
        .body(Body::from(r#"{"reason":"重新下单"}"#))
        .unwrap();
    let cancel_unpaid_res = app.clone().oneshot(cancel_unpaid_req).await.unwrap();
    let cancel_unpaid_body = to_bytes(cancel_unpaid_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let cancel_unpaid_value: Value = serde_json::from_slice(&cancel_unpaid_body).unwrap();
    assert_eq!(cancel_unpaid_value["success"], true);
    assert_eq!(cancel_unpaid_value["data"]["pay_status"], "UNPAID");

    let repurchase_req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/orders/{}/repurchase", unpaid_order_id))
        .header("authorization", user_auth.clone())
        .body(Body::empty())
        .unwrap();
    let repurchase_res = app.clone().oneshot(repurchase_req).await.unwrap();
    let repurchase_body = to_bytes(repurchase_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let repurchase_value: Value = serde_json::from_slice(&repurchase_body).unwrap();
    assert_eq!(repurchase_value["success"], true);
    assert_eq!(repurchase_value["data"]["status"], "PENDING_PAY");

    let create_order_admin_payload = json!({
        "store_id": store_id.to_string(),
        "delivery_type": "DELIVERY",
        "items": [{
            "product_id": product_id.to_string(),
            "title_snapshot": "椰子水",
            "price_snapshot": 990,
            "qty": 1
        }],
        "distance_km": 2.2,
        "address_id": address_id
    });
    let create_order_admin_req = Request::builder()
        .method("POST")
        .uri("/api/v1/orders/create")
        .header("content-type", "application/json")
        .header("authorization", user_auth.clone())
        .body(Body::from(create_order_admin_payload.to_string()))
        .unwrap();
    let create_order_admin_res = app.clone().oneshot(create_order_admin_req).await.unwrap();
    let create_order_admin_body = to_bytes(create_order_admin_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let create_order_admin_value: Value = serde_json::from_slice(&create_order_admin_body).unwrap();
    assert_eq!(create_order_admin_value["success"], true);
    let order_id_for_admin = create_order_admin_value["data"]["order_id"]
        .as_str()
        .unwrap()
        .to_string();

    let pay_order_admin_req = Request::builder()
        .method("POST")
        .uri("/api/v1/orders/pay")
        .header("content-type", "application/json")
        .header("authorization", user_auth.clone())
        .body(Body::from(
            json!({"order_id": order_id_for_admin}).to_string(),
        ))
        .unwrap();
    let pay_order_admin_res = app.clone().oneshot(pay_order_admin_req).await.unwrap();
    let pay_order_admin_body = to_bytes(pay_order_admin_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let pay_order_admin_value: Value = serde_json::from_slice(&pay_order_admin_body).unwrap();
    assert_eq!(pay_order_admin_value["success"], true);
    assert_eq!(pay_order_admin_value["data"]["status"], "PENDING_ACCEPT");

    let admin_accept_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/admin/v1/orders/{}/accept",
            order_id_for_admin
        ))
        .header("authorization", admin_auth.clone())
        .body(Body::empty())
        .unwrap();
    let admin_accept_res = app.clone().oneshot(admin_accept_req).await.unwrap();
    let admin_accept_body = to_bytes(admin_accept_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let admin_accept_value: Value = serde_json::from_slice(&admin_accept_body).unwrap();
    assert_eq!(admin_accept_value["success"], true);
    assert_eq!(admin_accept_value["data"]["status"], "ACCEPTED");

    let create_runner_payload = json!({
        "store_id": store_id.to_string(),
        "express_company": "顺丰",
        "pickup_code": "12-3-4567",
        "delivery_address": "A101",
        "receiver_name": "张三",
        "receiver_phone": "13800000000",
        "distance_km": 3.8
    });
    let create_runner_req = Request::builder()
        .method("POST")
        .uri("/api/v1/runner_orders/create")
        .header("content-type", "application/json")
        .header("authorization", user_auth.clone())
        .body(Body::from(create_runner_payload.to_string()))
        .unwrap();
    let create_runner_res = app.clone().oneshot(create_runner_req).await.unwrap();
    let create_runner_body = to_bytes(create_runner_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let create_runner_value: Value = serde_json::from_slice(&create_runner_body).unwrap();
    assert_eq!(create_runner_value["success"], true);

    let runner_order_id = create_runner_value["data"]["runner_order_id"]
        .as_str()
        .unwrap()
        .to_string();

    let pay_runner_req = Request::builder()
        .method("POST")
        .uri("/api/v1/runner_orders/pay")
        .header("content-type", "application/json")
        .header("authorization", user_auth)
        .body(Body::from(
            json!({"runner_order_id": runner_order_id}).to_string(),
        ))
        .unwrap();
    let pay_runner_res = app.clone().oneshot(pay_runner_req).await.unwrap();
    let pay_runner_body = to_bytes(pay_runner_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let pay_runner_value: Value = serde_json::from_slice(&pay_runner_body).unwrap();
    assert_eq!(pay_runner_value["success"], true);
    assert_eq!(pay_runner_value["data"]["status"], "PENDING_ACCEPT");

    let admin_accept_runner_req = Request::builder()
        .method("POST")
        .uri(format!(
            "/api/admin/v1/runner_orders/{}/accept",
            runner_order_id
        ))
        .header("authorization", admin_auth)
        .body(Body::empty())
        .unwrap();
    let admin_accept_runner_res = app.oneshot(admin_accept_runner_req).await.unwrap();
    let admin_accept_runner_body = to_bytes(admin_accept_runner_res.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let admin_accept_runner_value: Value =
        serde_json::from_slice(&admin_accept_runner_body).unwrap();
    assert_eq!(admin_accept_runner_value["success"], true);
    assert_eq!(admin_accept_runner_value["data"]["status"], "PROCESSING");
}
