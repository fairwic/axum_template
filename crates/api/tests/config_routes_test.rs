use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    body::{to_bytes, Body},
    http::Request,
};
use axum_api::auth::jwt::{encode_token, Claims};
use axum_api::state::BizConfig;
use axum_api::{create_router, AppState};
use axum_application::{
    AdminService, CartService, CategoryService, ProductService, StoreService, UserService,
};
use axum_common::AppResult;
use axum_domain::admin::entity::Admin;
use axum_domain::admin::repo::AdminRepository;
use axum_domain::cart::entity::Cart;
use axum_domain::cart::repo::CartRepository;
use axum_domain::category::entity::Category;
use axum_domain::category::repo::CategoryRepository;
use axum_domain::product::entity::Product;
use axum_domain::product::repo::ProductRepository;
use axum_domain::store::entity::Store;
use axum_domain::store::repo::StoreRepository;
use axum_domain::user::repo::UserRepository;
use axum_domain::User;
use chrono::Utc;
use serde_json::Value;
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
}

#[derive(Default)]
struct InMemoryCategoryRepo;

#[async_trait]
impl CategoryRepository for InMemoryCategoryRepo {
    async fn list_by_store(&self, _store_id: Ulid) -> AppResult<Vec<Category>> {
        Ok(vec![])
    }

    async fn create(&self, category: &Category) -> AppResult<Category> {
        Ok(category.clone())
    }
}

#[derive(Default)]
struct InMemoryProductRepo;

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
        Ok(product.clone())
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
struct FakeLbs;

#[async_trait]
impl axum_application::services::store_service::LbsService for FakeLbs {
    async fn distance_km(&self, _from: (f64, f64), _to: (f64, f64)) -> AppResult<f64> {
        Ok(0.0)
    }
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

#[tokio::test]
async fn test_get_config() {
    let user_repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepo::default());
    let admin_repo: Arc<dyn AdminRepository> = Arc::new(InMemoryAdminRepo::default());
    let store_repo: Arc<dyn StoreRepository> = Arc::new(InMemoryStoreRepo::default());
    let category_repo: Arc<dyn CategoryRepository> = Arc::new(InMemoryCategoryRepo);
    let product_repo: Arc<dyn ProductRepository> = Arc::new(InMemoryProductRepo);
    let cart_repo: Arc<dyn CartRepository> = Arc::new(InMemoryCartRepo::default());

    let state = AppState::new(
        UserService::new(user_repo),
        AdminService::new(admin_repo),
        StoreService::new(store_repo, Arc::new(FakeLbs)),
        CategoryService::new(category_repo),
        ProductService::new(product_repo),
        CartService::new(cart_repo),
        "secret".into(),
        3600,
        300,
    )
    .with_biz_config(BizConfig {
        delivery_free_radius_km: 3.0,
        runner_service_fee: 200,
        customer_service_phone: "13800138000".into(),
        runner_banner_enabled: true,
        runner_banner_text: "顺路代取快递".into(),
        pay_timeout_secs: 900,
        auto_accept_secs: 300,
    });
    let app = create_router(state);

    let req = Request::builder()
        .method("GET")
        .uri("/api/v1/config")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    let body = to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let value: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(value["success"], true);
    assert_eq!(value["data"]["delivery_free_radius_km"], 3.0);
    assert_eq!(value["data"]["runner_service_fee"], 200);
    assert_eq!(value["data"]["customer_service_phone"], "13800138000");
    assert_eq!(value["data"]["pay_timeout_secs"], 900);
    assert_eq!(value["data"]["auto_accept_secs"], 300);
}

#[tokio::test]
async fn test_admin_update_config_and_public_get_latest() {
    let user_repo: Arc<dyn UserRepository> = Arc::new(InMemoryUserRepo::default());
    let admin_repo: Arc<dyn AdminRepository> = Arc::new(InMemoryAdminRepo::default());
    let store_repo: Arc<dyn StoreRepository> = Arc::new(InMemoryStoreRepo::default());
    let category_repo: Arc<dyn CategoryRepository> = Arc::new(InMemoryCategoryRepo);
    let product_repo: Arc<dyn ProductRepository> = Arc::new(InMemoryProductRepo);
    let cart_repo: Arc<dyn CartRepository> = Arc::new(InMemoryCartRepo::default());

    let state = AppState::new(
        UserService::new(user_repo),
        AdminService::new(admin_repo),
        StoreService::new(store_repo, Arc::new(FakeLbs)),
        CategoryService::new(category_repo),
        ProductService::new(product_repo),
        CartService::new(cart_repo),
        "secret".into(),
        3600,
        300,
    )
    .with_biz_config(BizConfig {
        delivery_free_radius_km: 3.0,
        runner_service_fee: 200,
        customer_service_phone: "13800138000".into(),
        runner_banner_enabled: true,
        runner_banner_text: "顺路代取快递".into(),
        pay_timeout_secs: 900,
        auto_accept_secs: 300,
    });
    let app = create_router(state);

    let admin_auth = admin_auth_header();
    let update_req = Request::builder()
        .method("PUT")
        .uri("/api/admin/v1/config")
        .header("content-type", "application/json")
        .header("authorization", admin_auth)
        .body(Body::from(
            serde_json::json!({
                "delivery_free_radius_km": 4.0,
                "runner_service_fee": 300,
                "customer_service_phone": "400-123-4567",
                "runner_banner_enabled": false,
                "runner_banner_text": "代取服务升级",
                "pay_timeout_secs": 1200,
                "auto_accept_secs": 180
            })
            .to_string(),
        ))
        .unwrap();
    let update_res = app.clone().oneshot(update_req).await.unwrap();
    let update_body = to_bytes(update_res.into_body(), 1024 * 1024).await.unwrap();
    let update_value: Value = serde_json::from_slice(&update_body).unwrap();
    assert_eq!(update_value["success"], true);

    let get_req = Request::builder()
        .method("GET")
        .uri("/api/v1/config")
        .body(Body::empty())
        .unwrap();
    let get_res = app.oneshot(get_req).await.unwrap();
    let get_body = to_bytes(get_res.into_body(), 1024 * 1024).await.unwrap();
    let get_value: Value = serde_json::from_slice(&get_body).unwrap();

    assert_eq!(get_value["success"], true);
    assert_eq!(get_value["data"]["delivery_free_radius_km"], 4.0);
    assert_eq!(get_value["data"]["runner_service_fee"], 300);
    assert_eq!(get_value["data"]["customer_service_phone"], "400-123-4567");
    assert_eq!(get_value["data"]["runner_banner_enabled"], false);
    assert_eq!(get_value["data"]["runner_banner_text"], "代取服务升级");
    assert_eq!(get_value["data"]["pay_timeout_secs"], 1200);
    assert_eq!(get_value["data"]["auto_accept_secs"], 180);
}
